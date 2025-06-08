use std::sync::LazyLock;

use derive_more::{Debug, Display};
use rpl_meta::collect_elems_separated_by_comma;
use rpl_meta::symbol_table::{MetaVariableType, NonLocalMetaSymTab, WithPath};
use rpl_parser::generics::Choice2;
use rpl_parser::pairs::diagMessageInner;
use rpl_parser::{SpanWrapper, pairs};
use rustc_errors::{Applicability, LintDiagnostic};
use rustc_lint::{Level, Lint};
use rustc_middle::mir::Body;
use rustc_span::{Span, Symbol};
use sync_arena::declare_arena;
use thiserror::Error;

use super::Matched;
use crate::pat::{ConstVarIdx, TyVarIdx};

/// A dynamic error that can be used to report user-defined errors
///
/// See:
/// - [`rustc_errors::LintDiagnostic`].
/// - [`rustc_macros::diagnostics::lint_diagnostic_derive`].
/// - [`rustc_macros::LintDiagnostic`].
pub struct DynamicError {
    /// Primary message and its span.
    ///
    /// See [`rustc_errors::Diag::primary_message`].
    /// The primary message is the main error message that will be displayed to the user.
    primary: (String, Span),
    /// Label description, and the span of the label.
    ///
    /// See [`rustc_errors::Diag::span_label`].
    /// Labels are used to highlight specific parts of the code that are relevant to the error.
    labels: Vec<(String, Span)>,
    notes: Vec<(String, Option<Span>)>,
    helps: Vec<(String, Option<Span>)>,
    /// Suggestion description, alternative code, the span of the label, and its [`Applicability`].
    suggestions: Vec<(String, String, Span, Applicability)>,
    lint: &'static Lint,
}

impl LintDiagnostic<'_, ()> for DynamicError {
    fn decorate_lint(self, diag: &mut rustc_errors::Diag<'_, ()>) {
        let primary_message = self.primary.0;
        diag.primary_message(primary_message);
        for (label, span) in self.labels {
            diag.span_label(span, label);
        }
        for (help, span_help) in self.helps {
            if let Some(span_help) = span_help {
                diag.span_help(span_help, help);
            } else {
                diag.help(help);
            }
        }
        for (note, span_note) in self.notes {
            if let Some(span_note) = span_note {
                diag.span_note(span_note, note);
            } else {
                diag.note(note);
            }
        }
        for (suggestion, code, span, applicability) in self.suggestions {
            diag.span_suggestion(span, suggestion, code, applicability);
        }
    }
}

impl DynamicError {
    pub const fn primary_span(&self) -> Span {
        self.primary.1
    }
    /// Also see [`rustc_session::declare_tool_lint!`].
    pub const fn lint(&self) -> &'static Lint {
        self.lint
    }
    pub fn default_diagnostic(span: Span) -> Self {
        const LINT: Lint = Lint {
            name: "rpl::missing_diagnostic",
            default_level: Level::Deny,
            ..Lint::default_fields_for_macro()
        };
        let primary = (String::from("A pattern instance found in this span"), span);
        let labels = Vec::new();
        let notes = vec![
            (String::from("This is a fallback diagnostic message."), None),
            (
                String::from(
                    "You are seeing this because there is no corresponding diagnostic item in the RPL pattern file.",
                ),
                None,
            ),
        ];
        let helps = Vec::new();
        let suggestions = Vec::new();
        DynamicError {
            primary,
            labels,
            notes,
            helps,
            suggestions,
            lint: &LINT,
        }
    }
}

enum SubMsg<'i> {
    Str(&'i str),
    Ty(TyVarIdx),
    Const(ConstVarIdx),
}

impl<'i> SubMsg<'i> {
    fn parse(s: &pairs::diagMessageInner<'i, 0>, meta_vars: &NonLocalMetaSymTab) -> Vec<Self> {
        let mut msgs = Vec::new();
        for seg in s.iter_matched() {
            match seg {
                Choice2::_0(arg) => {
                    let (var_type, idx, _) = meta_vars
                        .get_from_symbol(Symbol::intern(arg.MetaVariable().span.as_str()))
                        .unwrap()
                        .expect_non_adt();
                    match var_type {
                        MetaVariableType::Type => msgs.push(SubMsg::Ty(idx.into())),
                        MetaVariableType::Const => msgs.push(SubMsg::Const(idx.into())),
                        MetaVariableType::Place => panic!(
                            "Unexpected place meta variable in diagnostic message: {}",
                            arg.span.as_str()
                        ),
                    }
                },
                Choice2::_1(text) => {
                    msgs.push(SubMsg::Str(text.span.as_str()));
                },
            }
        }
        msgs
    }
}

pub(crate) struct DynamicErrorBuilder<'i> {
    /// Primary message and its span.
    ///
    /// See [`DynamicError::primary`].
    primary: (Vec<SubMsg<'i>>, &'i str),
    /// Label description, and the span of the label.
    ///
    /// See [`DynamicError::labels`].
    labels: Vec<(Vec<SubMsg<'i>>, &'i str)>,
    /// Notes and their spans.
    ///
    /// See [`DynamicError::notes`].
    notes: Vec<(Vec<SubMsg<'i>>, Option<&'i str>)>,
    /// Helps and their spans.
    /// Helps are additional information that can help the user understand the error.
    ///
    /// See [`DynamicError::helps`].
    helps: Vec<(Vec<SubMsg<'i>>, Option<&'i str>)>,
    /// Suggestions, alternative code, their spans, and its [`Applicability`].
    ///
    /// See [`DynamicError::suggestions`].
    #[expect(clippy::type_complexity)]
    suggestions: Vec<(Vec<SubMsg<'i>>, Vec<SubMsg<'i>>, Option<&'i str>, Applicability)>,
    lint: &'static Lint,
}

declare_arena!(
    [
        [] _phantom: &'tcx (),
    ]
);

static ARENA: LazyLock<Arena<'static>> = LazyLock::new(Arena::default);

#[derive(Debug, Display, Error)]
pub enum ParseError<'i> {
    #[display("Primary message not found:\n{_0}")]
    PrimaryNotFound(SpanWrapper<'i>),
    #[display("Expected an identifier, but found:\n{_0}")]
    NotAnIdentifier(SpanWrapper<'i>),
    #[display("Expected a argument list, but found:\n{_0}")]
    #[expect(dead_code)]
    NotAnArgumentList(SpanWrapper<'i>),
    #[display("Expected a key-value pair, but found:\n{_0}")]
    #[expect(dead_code)]
    NotAKeyValuePair(SpanWrapper<'i>),
    #[display("No argument found:\n{_0}")]
    Empty(SpanWrapper<'i>),
    #[display("Too many arguments:\n{_0}")]
    TooManyArguments(SpanWrapper<'i>),
    #[display("Unexpected arguments:\n{_0}")]
    UnexpectedArguments(SpanWrapper<'i>),
    #[display("Invalid key {_0}:\n{_1}")]
    InvalidKey(&'i str, SpanWrapper<'i>),
    #[display("Missing {_0} in {_1}:\n{_2}")]
    MissingValue(&'static str, &'static str, SpanWrapper<'i>),
}

fn parse_ident<'i>(path: &'i std::path::Path, attrs: &pairs::diagAttrs<'i>) -> Result<&'i str, ParseError<'i>> {
    let (first, following, _trailing_comma) = attrs.get_matched();
    if !following.content.is_empty() {
        return Err(ParseError::TooManyArguments(SpanWrapper::new(attrs.span, path)));
    }
    let (ident, arguments_or_value) = first.get_matched();
    if arguments_or_value.is_some() {
        return Err(ParseError::NotAnIdentifier(SpanWrapper::new(first.span, path)));
    }
    Ok(ident.span.as_str())
}

fn parse_suggestion<'i>(
    path: &'i std::path::Path,
    attrs: &'i pairs::diagAttrs<'i>,
) -> Result<(&'i diagMessageInner<'i, 0>, Option<&'i str>, Applicability), ParseError<'i>> {
    let mut code = None;
    let mut span = None;
    let mut applicability = None;
    for attr in collect_elems_separated_by_comma!(attrs) {
        let key = attr.get_matched().0.span.as_str();
        if let Some(Choice2::_1(pair)) = attr.get_matched().1 {
            let (_, message) = pair.get_matched();
            match key {
                "code" => code = Some(message.diagMessageInner()),
                "span" => span = Some(message.diagMessageInner().span.as_str()),
                "applicability" => {
                    applicability = Some(match message.span.as_str() {
                        "machine_applicable" => Applicability::MachineApplicable,
                        "maybe_incorrect" => Applicability::MaybeIncorrect,
                        "has_placeholders" => Applicability::HasPlaceholders,
                        "unspecified" => Applicability::Unspecified,
                        _ => unimplemented!("Unrecognized applicability: {}", message.span.as_str()),
                    })
                },
                _ => return Err(ParseError::InvalidKey(key, SpanWrapper::new(attr.span, path))),
            }
        }
    }
    let code =
        code.ok_or_else(|| ParseError::MissingValue("code", "suggestion", SpanWrapper::new(attrs.span, path)))?;
    let applicability = applicability.unwrap_or(Applicability::Unspecified);
    Ok((code, span, applicability))
}

impl<'i> DynamicErrorBuilder<'i> {
    //FIXME: this function has a lot of `unwrap` calls, which can panic if the input is malformed.
    /// Create a [`DynamicErrorBuilder`] from a [`pairs::diagBlockItem`].
    pub(super) fn from_item(
        item: WithPath<'i, &'i pairs::diagBlockItem<'i>>,
        meta_vars: &NonLocalMetaSymTab,
    ) -> Result<Self, ParseError<'i>> {
        let path = item.path;
        let (_, _, _, diags, _, _) = item.get_matched();
        let mut primary = None;
        let mut labels = Vec::new();
        let mut notes = Vec::new();
        let mut helps = Vec::new();
        let mut suggestions = Vec::new();
        let mut level = Level::Deny;
        let mut name = None;

        for diag in collect_elems_separated_by_comma!(diags) {
            let (key, args, _, message) = diag.get_matched();

            let message = message.get_matched().1;

            let args = args.as_ref().map(|args| args.get_matched().1);

            let key = key.span.as_str();

            match key {
                "primary" => {
                    let ident = parse_ident(
                        path,
                        args.ok_or_else(|| ParseError::Empty(SpanWrapper::new(diag.span, path)))?,
                    )?;
                    primary = Some((SubMsg::parse(message, meta_vars), ident));
                },
                "label" => {
                    let ident = parse_ident(
                        path,
                        args.ok_or_else(|| ParseError::Empty(SpanWrapper::new(diag.span, path)))?,
                    )?;
                    labels.push((SubMsg::parse(message, meta_vars), ident));
                },
                "note" => {
                    let ident = args.map(|args| parse_ident(path, args)).transpose()?;
                    notes.push((SubMsg::parse(message, meta_vars), ident));
                },
                "help" => {
                    let ident = args.map(|args| parse_ident(path, args)).transpose()?;
                    helps.push((SubMsg::parse(message, meta_vars), ident));
                },
                "name" => {
                    if args.is_some() {
                        return Err(ParseError::UnexpectedArguments(SpanWrapper::new(diag.span, path)));
                    }
                    name = Some(message);
                },
                "level" => {
                    let message = message.span.as_str();
                    level = match message {
                        "allow" => Level::Allow,
                        "warn" => Level::Warn,
                        "deny" => Level::Deny,
                        "forbid" => Level::Forbid,
                        _ => unimplemented!("Unrecognized level: {message}",),
                    };
                },
                "suggestion" => {
                    let args = args.ok_or_else(|| ParseError::Empty(SpanWrapper::new(diag.span, path)))?;
                    let (code, span, applicability) = parse_suggestion(path, args)?;
                    let code = SubMsg::parse(code, meta_vars);
                    let message = SubMsg::parse(message, meta_vars);
                    suggestions.push((message, code, span, applicability));
                },
                _ => unimplemented!("Unrecognized key: {key:?}"),
            }
        }
        let primary = primary.ok_or_else(|| ParseError::PrimaryNotFound(SpanWrapper::new(item.span, path)))?;
        let name = name.unwrap().span.as_str();
        let builder = DynamicErrorBuilder {
            primary,
            labels,
            notes,
            helps,
            suggestions,
            lint: ARENA.alloc(Lint {
                name: ARENA.alloc_str(&format!("rpl::{name}")),
                default_level: level,
                ..Lint::default_fields_for_macro()
            }),
        };
        Ok(builder)
    }
    pub(crate) fn build<'tcx>(&self, body: &Body<'tcx>, matched: &impl Matched<'tcx>) -> DynamicError {
        fn format<'tcx>(message: &Vec<SubMsg>, matched: &impl Matched<'tcx>) -> String {
            let mut s = String::new();
            for msg in message {
                match msg {
                    SubMsg::Str(smsg) => s.push_str(smsg),
                    SubMsg::Ty(idx) => {
                        let ty = matched.type_meta_var(*idx);
                        s.push_str(&ty.to_string());
                    },
                    SubMsg::Const(idx) => {
                        let const_ = matched.const_meta_var(*idx);
                        s.push_str(&const_.to_string());
                    },
                }
            }
            s
        }
        let primary = (format(&self.primary.0, matched), matched.span(body, self.primary.1));
        let labels = self
            .labels
            .iter()
            .map(|(label, span)| (format(label, matched), matched.span(body, span)))
            .collect();
        let notes = self
            .notes
            .iter()
            .map(|(note, span)| (format(note, matched), span.map(|span| matched.span(body, span))))
            .collect();
        let helps = self
            .helps
            .iter()
            .map(|(help, span)| (format(help, matched), span.map(|span| matched.span(body, span))))
            .collect();
        let suggestions = self
            .suggestions
            .iter()
            .map(|(suggestion, code, span, applicability)| {
                (
                    format(suggestion, matched),
                    format(code, matched),
                    matched.span(body, span.unwrap()),
                    *applicability,
                )
            })
            .collect();
        let lint = self.lint;
        DynamicError {
            primary,
            labels,
            notes,
            helps,
            suggestions,
            lint,
        }
    }
}
