#![feature(rustc_private)]

pub mod error;
pub mod parser;
pub mod position;
pub mod span;

extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_span;

use std::path::Path;

pub use error::ParseError;
pub use parser::{Grammar, Rule, generics, pairs, rules};
use pest::Stack;
use pest_typed::tracker::Tracker;
pub use position::PositionWrapper;
pub use span::SpanWrapper;

pub fn parse<'i, T: pest_typed::ParsableTypedNode<'i, Rule>>(
    input: impl pest_typed::AsInput<'i>,
    path: &'i Path,
) -> Result<T, ParseError<'i>> {
    let input = input.as_input();
    let mut stack = Stack::new();

    let mut tracker = Tracker::new(input);
    match T::try_parse_with(input, &mut stack, &mut tracker) {
        Some(res) => Ok(res),
        None => Err(ParseError::new(tracker, path)),
    }
}

/// Parse input to [main](pairs::main).
pub fn parse_main<'i>(input: &'i str, path: &'i Path) -> Result<pairs::main<'i>, ParseError<'i>> {
    parse(input, path)
}
