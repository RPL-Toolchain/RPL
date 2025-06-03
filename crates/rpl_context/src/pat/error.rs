use std::sync::LazyLock;

use rpl_meta::symbol_table::{MetaVariableType, NonLocalMetaSymTab};
use rpl_parser::generics::Choice2;
use rpl_parser::pairs;
use rustc_errors::LintDiagnostic;
use rustc_lint::{Level, Lint};
use rustc_middle::mir::Body;
use rustc_span::{Span, Symbol};
use sync_arena::declare_arena;

use crate::pat::{ConstVarIdx, TyVarIdx};

use super::{LabelMap, Matched};

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
                    let (var_type, (idx, _)) = meta_vars
                        .get_from_symbol(Symbol::intern(arg.MetaVariable().span.as_str()))
                        .unwrap();
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
    primary: (Vec<SubMsg<'i>>, &'i str),
    /// Label description, and the span of the label.
    labels: Vec<(Vec<SubMsg<'i>>, &'i str)>,
    /// Notes and their spans.
    notes: Vec<(Vec<SubMsg<'i>>, Option<&'i str>)>,
    /// Helps and their spans.
    /// Helps are additional information that can help the user understand the error.
    helps: Vec<(Vec<SubMsg<'i>>, Option<&'i str>)>,
    lint: &'static Lint,
}

declare_arena!(
    [
        [] _phantom: &'tcx (),
    ]
);

static ARENA: LazyLock<Arena<'static>> = LazyLock::new(Arena::default);

impl<'i> DynamicErrorBuilder<'i> {
    //FIXME: this function has a lot of `unwrap` calls, which can panic if the input is malformed.
    /// Create a [`DynamicErrorBuilder`] from a [`pairs::diagBlockItem`].
    pub(super) fn from_item(item: &'i pairs::diagBlockItem<'i>, meta_vars: &NonLocalMetaSymTab) -> Self {
        let (_, _, _, pairs, _, _) = item.get_matched();
        let mut primary = (None, None);
        let mut labels = Vec::new();
        let mut notes = Vec::new();
        let mut helps = Vec::new();
        let mut level = Level::Deny;
        let mut name = None;

        for pair in pairs.iter_matched() {
            let (key, span, _, message, _) = pair.get_matched();

            let message = message.get_matched().1;

            let key = key.span.as_str();

            let span_name = span.as_ref().map(|span| span.get_matched().1.span.as_str());

            match key {
                "primary" => {
                    primary.0 = Some(SubMsg::parse(message, meta_vars));
                    if let Some(span_name) = span_name {
                        primary.1 = Some(span_name);
                    }
                },
                "label" => {
                    labels.push((SubMsg::parse(message, meta_vars), span_name.unwrap()));
                },
                "note" => {
                    notes.push((SubMsg::parse(message, meta_vars), span_name));
                },
                "help" => {
                    helps.push((SubMsg::parse(message, meta_vars), span_name));
                },
                "name" => {
                    name = Some(message);
                },
                "level" => {
                    let message = message.span.as_str();
                    level = match message {
                        "allow" => Level::Allow,
                        "warning" => Level::Warn,
                        "deny" => Level::Deny,
                        "forbid" => Level::Forbid,
                        _ => unimplemented!("Unrecognized level: {message}",),
                    };
                },
                _ => unimplemented!("Unrecognized key: {key:?}"),
            }
        }
        let primary = (primary.0.unwrap(), primary.1.unwrap());
        let name = name.unwrap().span.as_str();
        let builder = DynamicErrorBuilder {
            primary,
            labels,
            notes,
            helps,
            lint: ARENA.alloc(Lint {
                name: ARENA.alloc_str(&format!("rpl::{name}")),
                default_level: level,
                ..Lint::default_fields_for_macro()
            }),
        };
        builder
    }
    pub(crate) fn build<'tcx>(
        &self,
        label_map: &LabelMap,
        body: &Body<'tcx>,
        matched: &impl Matched<'tcx>,
    ) -> DynamicError {
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
        let primary = (
            format(&self.primary.0, matched),
            matched.span(label_map, body, self.primary.1),
        );
        let labels = self
            .labels
            .iter()
            .map(|(label, span)| (format(label, matched), matched.span(label_map, body, span)))
            .collect();
        let notes = self
            .notes
            .iter()
            .map(|(note, span)| {
                (
                    format(note, matched),
                    span.map(|span| matched.span(label_map, body, span)),
                )
            })
            .collect();
        let helps = self
            .helps
            .iter()
            .map(|(help, span)| {
                (
                    format(help, matched),
                    span.map(|span| matched.span(label_map, body, span)),
                )
            })
            .collect();
        let lint = self.lint;
        DynamicError {
            primary,
            labels,
            notes,
            helps,
            lint,
        }
    }
}
