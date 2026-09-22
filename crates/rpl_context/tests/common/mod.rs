//! Shared boilerplate for ops integration tests.
//!
//! Every test in this crate needs a `&'static MetaContext<'static>` so that
//! `PatternCtxt::entered_no_tcx` (which takes a re-entrant closure with no
//! bound lifetime) can accept it. The mechanical way to get that is
//! `Box::leak` plus a `'static`-lifetimed arena.
//!
//! Before this module existed, every test (8 occurrences across 4 files)
//! repeated the same 5-line boilerplate. This module collapses it to
//! `make_static_mctx("test.rpl", src)`.
//!
//! The arena is created once per integration-test binary via [`LazyLock`] and
//! shared across all `#[test]` functions in that binary. Each test still
//! parses its own source string and gets its own `MetaContext`; the savings
//! come from not creating (and leaking) a fresh `Arena` per test.

#![allow(dead_code)] // not every test consumes every helper

use std::path::PathBuf;
use std::sync::LazyLock;

use rpl_meta::RPLMetaError;
use rpl_meta::arena::Arena;
use rpl_meta::context::MetaContext;

/// Returns a shared, lazily-initialised `&'static Arena<'static>` for the
/// current integration-test binary. All tests in a binary share one arena;
/// the underlying allocation grows but is never freed (test process exit
/// reclaims it).
pub fn shared_arena() -> &'static Arena<'static> {
    static ARENA: LazyLock<&'static Arena<'static>> = LazyLock::new(|| Box::leak(Box::<Arena<'static>>::default()));
    *ARENA
}

/// Parse `src` and return a `'static` [`MetaContext`].
///
/// Panics on any parse/collect error — use this when the test fixture is
/// intentionally well-formed and a failure indicates a fixture bug.
pub fn make_static_mctx(filename: &str, src: &str) -> &'static MetaContext<'static> {
    make_static_mctx_with(filename, src, |err| {
        panic!("RPL parse/collect error: {err}");
    })
}

/// Like [`make_static_mctx`] but lets the caller supply a custom error
/// handler.
///
/// Used by tests that intentionally feed malformed input — e.g. R6
/// fixtures that reference an undeclared pattern-level meta-var and need to
/// suppress the `NonLocalMetaVariableNotDeclared` error from
/// `parse_and_collect` so the R6 check itself can run.
pub fn make_static_mctx_with(
    filename: &str,
    src: &str,
    handler: impl FnMut(&RPLMetaError<'static>),
) -> &'static MetaContext<'static> {
    let path_and_content: &'static Vec<(PathBuf, String)> =
        Box::leak(Box::new(vec![(PathBuf::from(filename), src.to_string())]));
    Box::leak(Box::new(rpl_meta::parse_and_collect(
        shared_arena(),
        path_and_content,
        handler,
    )))
}
