use crate::SpanWrapper;
use crate::parser::Rule;
use crate::position::PositionWrapper;
use colored::Colorize;
use derive_more::derive::Debug;
use pest_typed::tracker::{SpecialError, Tracker};
use pest_typed::{Input, Position};
use rustc_errors::{Diag, DiagCtxtHandle, Diagnostic, Level};
use rustc_span::ErrorGuaranteed;
use std::collections::BTreeMap;
use std::convert::identity;
use std::fmt::{Display, Write};
use std::path::Path;

fn format_rule(rule: Option<Rule>, f: &mut impl Write) -> std::fmt::Result {
    match rule {
        None => write!(f, "Root Rule")?,
        Some(rule) => write!(f, "{:?}", rule)?,
    }
    Ok(())
}

fn format_tracker<'i>(
    path: &'i Path,
    pos: Position<'i>,
    rules: &BTreeMap<Option<Rule>, Tracked<Rule>>,
    replacer: impl FnOnce(pest_typed::Position<'i>) -> pest_typed::Position<'i>,
    f: &mut impl Write,
) -> Result<(), std::fmt::Error> {
    let pos = replacer(pos);
    write!(f, "{}", PositionWrapper::new(pos, path))?;
    let log10 = {
        let mut n = pos.line_col().0;
        let mut i = 1;
        while n >= 10 {
            n /= 10;
            i += 1;
        }
        i
    };

    write!(f, "\n{} {}", " ".repeat(log10), "Possible RPL grammar rules: ".blue())?;
    write!(f, "[")?;
    let mut iter = rules.keys().cloned();
    if let Some(rule) = iter.next() {
        format_rule(rule, f)?;
    }
    for rule in iter {
        write!(f, ", ")?;
        format_rule(rule, f)?;
    }
    write!(f, "]")?;

    Ok(())
}

type Tracked<R> = (Vec<R>, Vec<R>, Vec<SpecialError>);

/// Errors from parser.
#[derive(Debug)]
#[debug("ParseError({path:?})")]
pub struct ParseError<'i> {
    path: &'i Path,
    position: Position<'i>,
    attempts: BTreeMap<Option<Rule>, Tracked<Rule>>,
}

impl<'i> ParseError<'i> {
    /// Create a [ParseError].'
    pub fn new(tracker: Tracker<'i, Rule>, path: &'i Path) -> Self {
        let (pos, rules) = tracker.finish();
        Self {
            path,
            position: pos,
            attempts: rules,
        }
    }
}

impl Display for ParseError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format_tracker(self.path, self.position, &self.attempts, identity, f)
    }
}
impl std::error::Error for ParseError<'_> {}

struct Attempts<'e>(&'e BTreeMap<Option<Rule>, Tracked<Rule>>);

impl Display for Attempts<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        let mut iter = self.0.keys();
        if let Some(rule) = iter.next() {
            format_rule(*rule, f)?;
        }
        for rule in iter {
            write!(f, ", ")?;
            format_rule(*rule, f)?;
        }
        write!(f, "]")?;
        Ok(())
    }
}

impl Diagnostic<'_, ErrorGuaranteed> for &ParseError<'_> {
    fn into_diag(self, dcx: DiagCtxtHandle<'_>, _: Level) -> Diag<'_, ErrorGuaranteed> {
        let span = self.position.span(
            &Position::new(
                self.position.input(),
                self.position.pos()
                    + self.position.input()[self.position.pos()..]
                        .chars()
                        .next()
                        .map(|c| c.len_utf8())
                        .unwrap_or(0),
            )
            .unwrap(),
        );
        dcx.struct_err("Parse error")
            .with_help(format!("Possible RPL grammar rules: {}", Attempts(&self.attempts)))
            .with_help(SpanWrapper::new(span, self.path).to_string())
    }
}

// //FIXME: the source file it uses is incorrect
// pub fn span_cvt(span: SpanWrapper<'_>) -> rustc_span::Span {
//     let expn_id = LocalExpnId::fresh_empty();
//     let expn_data = ExpnData::default(
//         ExpnKind::AstPass(AstPass::StdImports),
//         Span::new(BytePos(0), BytePos(0), SyntaxContext::root(), None),
//         LATEST_STABLE_EDITION,
//         None,
//         None,
//     );
//     expn_id.set_expn_data(expn_data, ctx);
//     let expn_id = expn_id.to_expn_id();
//     let ctx = SyntaxContext::root().apply_mark(expn_id, Transparency::Opaque);
//     rustc_span::Span::new(
//         rustc_span::BytePos(span.inner().start().try_into().unwrap()),
//         rustc_span::BytePos(span.inner().end().try_into().unwrap()),
//         ctx,
//         None,
//     )
// }
