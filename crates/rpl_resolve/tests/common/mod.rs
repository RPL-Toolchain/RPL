//! Shared boilerplate for ops integration tests.
//!
//! Mirrors `crates/rpl_context/tests/common/mod.rs`. See that file for the
//! design rationale. Duplicated here because Cargo integration tests in
//! different crates are independent binaries and cannot share a `tests/`
//! sub-module across crate boundaries without a dedicated dev-dep crate
//! (which would be heavier than the ~50 LOC duplicated below).

#![allow(dead_code)] // not every test consumes every helper

use std::path::PathBuf;
use std::sync::LazyLock;

use rpl_meta::RPLMetaError;
use rpl_meta::arena::Arena;
use rpl_meta::context::MetaContext;

/// Returns a shared, lazily-initialised `&'static Arena<'static>` for the
/// current integration-test binary.
pub fn shared_arena() -> &'static Arena<'static> {
    static ARENA: LazyLock<&'static Arena<'static>> = LazyLock::new(|| Box::leak(Box::<Arena<'static>>::default()));
    *ARENA
}

/// Parse `src` and return a `'static` [`MetaContext`]. Panics on any
/// parse/collect error.
pub fn make_static_mctx(filename: &str, src: &str) -> &'static MetaContext<'static> {
    make_static_mctx_with(filename, src, |err| {
        panic!("RPL parse/collect error: {err}");
    })
}

/// Like [`make_static_mctx`] but lets the caller supply a custom error
/// handler. Used by R6 fixtures that intentionally feed malformed input.
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
