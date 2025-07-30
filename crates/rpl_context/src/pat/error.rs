// #![warn(
//     clippy::unreachable,
//     clippy::todo,
//     clippy::panic,
//     clippy::unwrap_used,
//     clippy::unimplemented,
//     clippy::manual_assert,
//     clippy::missing_panics_doc
// )]

use derive_more::{Debug, Display};
use rpl_meta::symbol_table::{DiagSymbolTable, MetaVariableType, NonLocalMetaSymTab, WithPath};
use rpl_meta::{DYNAMIC, collect_elems_separated_by_comma};
use rpl_parser::generics::Choice2;
use rpl_parser::pairs::diagMessageInner;
use rpl_parser::{SpanWrapper, pairs};
use rustc_data_structures::fx::{FxHashMap, FxHashSet};
use rustc_errors::{Applicability, LintDiagnostic, MultiSpan};
use rustc_hir::FnDecl;
use rustc_lint::{Level, Lint};
use rustc_middle::mir::Body;
use rustc_span::source_map::SourceMap;
use rustc_span::{Span, Symbol};
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
    primary: (String, MultiSpan),
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

impl LintDiagnostic<'_, ()> for Box<DynamicError> {
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
    // const fn attr_error(span: Span) -> DynamicError {
    //     DynamicError {
    //         primary: (Cow::Borrowed("Ill-formed RPL dynamic attribute"), span),
    //         labels: Vec::new(),
    //         notes: Vec::new(),
    //         helps: Vec::new(),
    //     }
    // }
    fn unknown_attribute_error(span: Span) -> Self {
        Self {
            primary: ("Unknown attribute key".to_string(), MultiSpan::from_span(span)),
            labels: Vec::new(),
            notes: vec![(
                "Allowed attribute keys are: `primary_message`, `labels`, `note`, `help`".to_string(),
                None,
            )],
            helps: Vec::new(),
            suggestions: Vec::new(),
            lint: DYNAMIC,
        }
    }
    fn missing_primary_message_error(attr: &rustc_hir::Attribute) -> Self {
        Self {
            primary: ("Missing primary message".to_string(), MultiSpan::from_span(attr.span)),
            labels: Vec::new(),
            notes: Vec::new(),
            helps: Vec::new(),
            suggestions: Vec::new(),
            lint: DYNAMIC,
        }
    }
    fn item_to_value_str(item: &rustc_ast::MetaItemInner) -> Result<Symbol, Box<Self>> {
        item.value_str().ok_or_else(|| {
            // If the value is not a string, we return an error.
            // This is a fallback to ensure that we always return a valid error.
            Self {
                primary: ("Expected a string value".to_string(), MultiSpan::from_span(item.span())),
                labels: Vec::new(),
                notes: Vec::new(),
                helps: Vec::new(),
                suggestions: Vec::new(),
                lint: DYNAMIC,
            }
            .into()
        })
    }
    fn expected_meta_item_list_error(span: Span) -> Box<Self> {
        Self {
            primary: ("Expected a meta item list".to_string(), MultiSpan::from_span(span)),
            labels: Vec::new(),
            notes: Vec::new(),
            helps: Vec::new(),
            suggestions: Vec::new(),
            lint: DYNAMIC,
        }
        .into()
    }
    fn attr_to_meta_item_list(
        attr: &rustc_hir::Attribute,
    ) -> Result<impl Iterator<Item = rustc_ast::MetaItemInner>, Box<Self>> {
        attr.meta_item_list().map_or_else(
            || Err(Self::expected_meta_item_list_error(attr.span())),
            |vec| Ok(vec.into_iter()),
        )
    }
    fn item_to_meta_item_list(
        item: &rustc_ast::MetaItemInner,
    ) -> Result<impl Iterator<Item = &rustc_ast::MetaItemInner>, Box<Self>> {
        item.meta_item_list().map_or_else(
            || Err(Self::expected_meta_item_list_error(item.span())),
            |vec| Ok(vec.iter()),
        )
    }
    fn from_attr_impl(attr: &rustc_hir::Attribute, span: Span) -> Result<Box<DynamicError>, Box<DynamicError>> {
        let items = Self::attr_to_meta_item_list(attr)?;
        let mut primary_message = None;
        let mut labels = Vec::new();
        let mut notes = Vec::new();
        let mut helps = Vec::new();
        for item in items {
            match item.name_or_empty().as_str() {
                "primary_message" => {
                    primary_message = Some(Self::item_to_value_str(&item)?.to_string());
                },
                "labels" => {
                    let label_list = Self::item_to_meta_item_list(&item)?;
                    for label_item in label_list {
                        // FIXME: `label_item.span()` is not the actual span it refers to,
                        labels.push((Self::item_to_value_str(label_item)?.to_string(), label_item.span()));
                    }
                },
                "note" => {
                    notes.push((Self::item_to_value_str(&item)?.to_string(), None));
                },
                "help" => {
                    helps.push((Self::item_to_value_str(&item)?.to_string(), None));
                },
                _ => {
                    // error!("Unknown attribute key {:?}", item.name_or_empty())
                    return Err(Self::unknown_attribute_error(item.span()).into());
                },
            }
        }
        let primary_message = primary_message.ok_or_else(|| Self::missing_primary_message_error(attr))?;
        let primary = (primary_message, MultiSpan::from_span(span));
        Ok(DynamicError {
            primary,
            labels,
            notes,
            helps,
            suggestions: Vec::new(),
            lint: DYNAMIC,
        }
        .into())
    }
    pub fn from_attr(attr: &rustc_hir::Attribute, span: Span) -> Box<DynamicError> {
        Self::from_attr_impl(attr, span).unwrap_or_else(|err| {
            // If we fail to parse the attribute, we return an error.
            // This is a fallback to ensure that we always return a valid error.
            err
        })
    }
}

impl DynamicError {
    pub const fn primary_span(&self) -> &MultiSpan {
        &self.primary.1
    }
    /// Also see [`rustc_session::declare_tool_lint!`].
    pub const fn lint(&self) -> &'static Lint {
        self.lint
    }
    pub fn default_diagnostic(pat_name: Symbol, span: Span) -> Self {
        const LINT: Lint = Lint {
            name: "rpl::missing_diagnostic",
            default_level: Level::Deny,
            ..Lint::default_fields_for_macro()
        };
        let primary = (
            String::from("A pattern instance found in this span"),
            MultiSpan::from_span(span),
        );
        let labels = Vec::new();
        let notes = vec![
            (String::from("This is a fallback diagnostic message."), None),
            (
                format!(
                    "You are seeing this because there is no corresponding diagnostic item for pattern {pat_name:?} in the RPL pattern file.",
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

#[derive(Debug)]
enum SubMsg<'i> {
    Str(&'i str),
    Ty(TyVarIdx),
    Const(ConstVarIdx),
    FnName,
    Label(Symbol),
}

impl<'i> SubMsg<'i> {
    fn parse<'mcx>(
        s: &pairs::diagMessageInner<'i, 0>,
        meta_vars: &NonLocalMetaSymTab<'mcx>,
        consts: &FxHashMap<Symbol, &'i str>,
        labels: &FxHashSet<Symbol>,
    ) -> Vec<Self> {
        let mut msgs = Vec::new();
        for seg in s.iter_matched() {
            match seg {
                Choice2::_0(arg) => {
                    let meta_var = arg.MetaVariable();
                    let name = meta_var.Word();
                    let meta_var = meta_var.span.as_str();
                    let name = Symbol::intern(name.span.as_str());
                    if let Some(const_value) = consts.get(&name) {
                        msgs.push(SubMsg::Str(const_value));
                    } else if meta_var == "$fn" {
                        // FIXME: this is a hack to handle `$fn` meta variable
                        msgs.push(SubMsg::FnName)
                    } else if labels.contains(&name) {
                        msgs.push(SubMsg::Label(name))
                    } else {
                        let (var_type, idx, _) = meta_vars
                            .get_meta_var_from_name(meta_var)
                            .unwrap_or_else(|| {
                                panic!(
                                    "Meta variable `{}` not found:\n    non-local meta symbol table {:?}\n    labels: {:?}",
                                    meta_var, meta_vars, labels
                                )
                            })
                            .expect_non_adt();
                        match var_type {
                            MetaVariableType::Type => msgs.push(SubMsg::Ty(idx.into())),
                            MetaVariableType::Const => msgs.push(SubMsg::Const(idx.into())),
                            MetaVariableType::Place => panic!(
                                "Unexpected place meta variable in diagnostic message: {}",
                                arg.span.as_str()
                            ),
                        }
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
    primary: (Vec<SubMsg<'i>>, Vec<&'i str>),
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
    suggestions: Vec<(Vec<SubMsg<'i>>, Vec<SubMsg<'i>>, &'i str, Applicability)>,
    lint: &'static Lint,
}

#[derive(Debug, Display, Error)]
pub enum ParseError<'i> {
    #[display("Primary message not found:\n{_0}")]
    PrimaryNotFound(SpanWrapper<'i>),
    #[display("Lint name not found:\n{_0}")]
    MissingName(SpanWrapper<'i>),
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
    #[display("Unrecognized enum {_0} {_1:?}:\n{_2}")]
    UnrecognizedEnum(&'static str, &'i str, SpanWrapper<'i>),
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

fn parse_idents<'i>(path: &'i std::path::Path, attrs: &pairs::diagAttrs<'i>) -> Result<Vec<&'i str>, ParseError<'i>> {
    let mut idents = Vec::new();
    for attr in collect_elems_separated_by_comma!(attrs) {
        let (ident, arguments_or_value) = attr.get_matched();
        if arguments_or_value.is_some() {
            return Err(ParseError::NotAnIdentifier(SpanWrapper::new(attr.span, path)));
        }
        idents.push(ident.span.as_str());
    }
    Ok(idents)
}
fn parse_suggestion<'i>(
    path: &'i std::path::Path,
    attrs: &'i pairs::diagAttrs<'i>,
) -> Result<(&'i diagMessageInner<'i, 0>, &'i str, Applicability), ParseError<'i>> {
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
                    let msg = message.diagMessageInner().span.as_str();
                    applicability = Some(match msg {
                        "machine_applicable" => Applicability::MachineApplicable,
                        "maybe_incorrect" => Applicability::MaybeIncorrect,
                        "has_placeholders" => Applicability::HasPlaceholders,
                        "unspecified" => Applicability::Unspecified,
                        _ => Err(ParseError::UnrecognizedEnum(
                            "applicability",
                            msg,
                            SpanWrapper::new(message.span, path),
                        ))?,
                    })
                },
                _ => return Err(ParseError::InvalidKey(key, SpanWrapper::new(attr.span, path))),
            }
        }
    }
    let code =
        code.ok_or_else(|| ParseError::MissingValue("code", "suggestion", SpanWrapper::new(attrs.span, path)))?;
    let span =
        span.ok_or_else(|| ParseError::MissingValue("span", "suggestion", SpanWrapper::new(attrs.span, path)))?;
    let applicability = applicability.unwrap_or(Applicability::Unspecified);
    Ok((code, span, applicability))
}

impl<'i> DynamicErrorBuilder<'i> {
    // FIXME: this function has a lot of `unwrap` calls, which can panic if the input is malformed.
    /// Create a [`DynamicErrorBuilder`] from a [`pairs::diagBlockItem`].
    ///
    /// # Note
    ///
    /// See [`rpl_meta::symbol_table::DiagSymbolTable`] for earlier passes.
    pub(super) fn from_item<'mcx: 'i>(
        item: WithPath<'i, &'i pairs::diagBlockItem<'i>>,
        meta_vars: &NonLocalMetaSymTab<'mcx>,
        consts: &FxHashMap<Symbol, &'i str>,
        locals: &FxHashSet<Symbol>,
        table: &DiagSymbolTable,
    ) -> Result<Self, ParseError<'i>> {
        let path = item.path;
        let (_, _, _, diags, _, _) = item.get_matched();
        let mut primary = None;
        let mut labels = Vec::new();
        let mut notes = Vec::new();
        let mut helps = Vec::new();
        let mut suggestions = Vec::new();
        let mut name = None;

        for diag in collect_elems_separated_by_comma!(diags) {
            let (key, args, _, message) = diag.get_matched();

            let message = message.get_matched().1;

            let args = args.as_ref().map(|args| args.get_matched().1);

            let key = key.span.as_str();

            match key {
                "primary" => {
                    let idents = parse_idents(
                        path,
                        args.ok_or_else(|| ParseError::Empty(SpanWrapper::new(diag.span, path)))?,
                    )?;
                    primary = Some((SubMsg::parse(message, meta_vars, consts, locals), idents));
                },
                "label" => {
                    let ident = parse_ident(
                        path,
                        args.ok_or_else(|| ParseError::Empty(SpanWrapper::new(diag.span, path)))?,
                    )?;
                    labels.push((SubMsg::parse(message, meta_vars, consts, locals), ident));
                },
                "note" => {
                    let ident = args.map(|args| parse_ident(path, args)).transpose()?;
                    notes.push((SubMsg::parse(message, meta_vars, consts, locals), ident));
                },
                "help" => {
                    let ident = args.map(|args| parse_ident(path, args)).transpose()?;
                    helps.push((SubMsg::parse(message, meta_vars, consts, locals), ident));
                },
                "name" => {
                    if args.is_some() {
                        return Err(ParseError::UnexpectedArguments(SpanWrapper::new(diag.span, path)));
                    }
                    name = Some(message);
                },
                "level" => (),
                "suggestion" => {
                    let args = args.ok_or_else(|| ParseError::Empty(SpanWrapper::new(diag.span, path)))?;
                    let (code, span, applicability) = parse_suggestion(path, args)?;
                    let code = SubMsg::parse(code, meta_vars, consts, locals);
                    let message = SubMsg::parse(message, meta_vars, consts, locals);
                    suggestions.push((message, code, span, applicability));
                },
                _ => Err(ParseError::InvalidKey(key, SpanWrapper::new(diag.span, path)))?,
            }
        }
        let primary = primary.ok_or_else(|| ParseError::PrimaryNotFound(SpanWrapper::new(item.span, path)))?;
        let name = name
            .ok_or_else(|| ParseError::MissingName(SpanWrapper::new(item.span, path)))?
            .span
            .as_str();
        let lint = table.get(name).unwrap_or_else(|| {
            panic!(
                "Lint `{}` not found in the symbol table:\n    symbol table: {:?}\n    locals: {:?}",
                name,
                table.lints(),
                locals
            )
        });
        // trace!(?primary, ?labels, ?notes, ?helps, ?suggestions);
        let builder = DynamicErrorBuilder {
            primary,
            labels,
            notes,
            helps,
            suggestions,
            lint,
        };
        Ok(builder)
    }
    pub(crate) fn build<'tcx>(
        &self,
        source_map: &SourceMap,
        fn_name: Option<Symbol>,
        body: &Body<'tcx>,
        decl: &FnDecl<'tcx>,
        matched: &impl Matched<'tcx>,
    ) -> DynamicError {
        struct Formatter<'a, 'tcx, M: Matched<'tcx>> {
            source_map: &'a SourceMap,
            fn_name: Option<Symbol>,
            body: &'a Body<'tcx>,
            decl: &'a FnDecl<'tcx>,
            matched: &'a M,
        }
        impl<'tcx, M: Matched<'tcx>> Formatter<'_, 'tcx, M> {
            fn format(&self, message: &Vec<SubMsg>) -> String {
                let mut s = String::new();
                for msg in message {
                    match msg {
                        SubMsg::Str(smsg) => s.push_str(smsg),
                        SubMsg::Ty(idx) => {
                            let ty = self.matched.type_meta_var(*idx);
                            s.push_str(&ty.to_string());
                        },
                        SubMsg::Const(idx) => {
                            let const_ = self.matched.const_meta_var(*idx);
                            s.push_str(&const_.to_string());
                        },
                        SubMsg::FnName => {
                            s.push_str(self.fn_name.unwrap().as_str());
                        },
                        SubMsg::Label(local) => {
                            let local_name = self.matched.span(self.body, self.decl, local.as_str());
                            s.push_str(&self.source_map.span_to_snippet(local_name).unwrap());
                        },
                    }
                }
                s
            }
        }
        let formatter = Formatter {
            source_map,
            fn_name,
            body,
            decl,
            matched,
        };
        let primary = (
            formatter.format(&self.primary.0),
            matched.multi_span(body, decl, &self.primary.1),
        );
        let labels = self
            .labels
            .iter()
            .map(|(label, span)| (formatter.format(label), matched.span(body, decl, span)))
            .collect();
        let notes = self
            .notes
            .iter()
            .map(|(note, span)| (formatter.format(note), span.map(|span| matched.span(body, decl, span))))
            .collect();
        let helps = self
            .helps
            .iter()
            .map(|(help, span)| (formatter.format(help), span.map(|span| matched.span(body, decl, span))))
            .collect();
        let suggestions = self
            .suggestions
            .iter()
            .map(|(suggestion, code, span, applicability)| {
                (
                    formatter.format(suggestion),
                    formatter.format(code),
                    matched.span(body, decl, span),
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
