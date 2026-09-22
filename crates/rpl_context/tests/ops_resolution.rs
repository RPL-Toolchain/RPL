//! Substitution + validation tests for `resolve_ops_config`.
//!
//! Covers the five scenarios from Task 9 of the abstract-ops plan:
//!   - Happy path: bindings expand cleanly, instance retained.
//!   - C2: missing meta-var binding → warning, instance skipped.
//!   - C4: unknown extra key → warning, instance skipped.
//!   - C7: cyclic substitution → warning, instance skipped.
//!   - Mixed good+bad instances: only the good one is retained.

#![allow(internal_features)]
#![feature(rustc_private)]
#![feature(rustc_attrs)]
#![feature(let_chains)]
#![feature(box_patterns)]
#![feature(debug_closure_helpers)]
#![recursion_limit = "256"]

extern crate rustc_data_structures;
extern crate rustc_span;

use std::collections::BTreeMap;

use rpl_config::RawOpInstance;
use rpl_context::PatternCtxt;
use rpl_context::pat::ops_resolved::{ResolveDiagnostic, resolve_ops_config};
use rustc_span::Symbol;

mod common;
use common::make_static_mctx;

// ---------------------------------------------------------------------------
// Helper utilities
// ---------------------------------------------------------------------------

/// Build a `RawOpInstance` from a list of free-placeholder names and key/value pairs.
fn instance(free: &[&str], pairs: &[(&str, &str)]) -> RawOpInstance {
    RawOpInstance {
        free: free.iter().map(|s| s.to_string()).collect(),
        bindings: pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<BTreeMap<_, _>>(),
    }
}

/// The RPL source shared by all tests — declares a `sync` group with two type
/// meta-vars ($T, $U) and two ops ($lock, $unlock).
const PATTERN_DECL: &str = r#"
pattern test
ops {
    sync[$T: type, $U: type] = {
        fn $lock(&mut $T) -> $U;
        fn $unlock(&mut $U) -> _;
    }
}
patt { p[] = fn _ () -> _ {} }
"#;

/// Parse the RPL source, lower it into a `Pattern`, then call the supplied
/// closure `f` with that pattern (inside `PatternCtxt::entered_no_tcx`).
///
/// The arena and `MetaContext` are obtained via the shared helper in
/// `tests/common/mod.rs`, which leaks them into `'static` once per binary.
fn with_pattern<F: for<'pcx> FnMut(&rpl_context::pat::Pattern<'pcx>)>(src: &str, mut f: F) {
    let mctx = make_static_mctx("test.rpl", src);
    PatternCtxt::entered_no_tcx(|pcx| {
        pcx.add_parsed_patterns(mctx);
        pcx.for_each_rpl_pattern(|_, pattern| {
            f(pattern);
        });
    });
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Happy path: all required bindings present, expansion converges, no residual
/// undeclared placeholders.  `$T`/`$U` get expanded into the concrete path
/// strings; `$1` is a free placeholder and is left intact.
#[test]
fn happy_path_eager_expansion() {
    with_pattern(PATTERN_DECL, |pattern| {
        let raw = vec![(
            "sync".to_string(),
            vec![instance(
                &["$1"],
                &[
                    ("T", "std::sync::Mutex<$1>"),
                    ("U", "std::sync::MutexGuard<$1>"),
                    ("lock", "$T::lock"),
                    ("unlock", "$U::drop"),
                ],
            )],
        )];
        let (cfg, diags) = resolve_ops_config(pattern, &raw);
        assert!(
            diags.is_empty(),
            "no diagnostics expected for happy path, got: {diags:?}"
        );

        let sync = cfg.instances_of("sync");
        assert_eq!(sync.len(), 1, "expected exactly one resolved instance");

        let inst = &sync[0];

        // After expansion, `lock` should contain "Mutex", "$1", and "lock".
        let lock_val = inst.paths.get(&Symbol::intern("lock")).unwrap();
        assert!(
            lock_val.contains("Mutex"),
            "expanded lock path should reference Mutex, got: {lock_val}"
        );
        assert!(
            lock_val.contains("$1"),
            "expanded lock path should retain free placeholder $1, got: {lock_val}"
        );
        assert!(
            lock_val.contains("lock"),
            "expanded lock path should contain 'lock', got: {lock_val}"
        );

        // T should expand to the Mutex string.
        let t_val = inst.types.get(&Symbol::intern("T")).unwrap();
        assert!(
            t_val.contains("Mutex"),
            "T type should expand to Mutex<$1>, got: {t_val}"
        );
        assert!(
            t_val.contains("$1"),
            "T type should retain $1 placeholder, got: {t_val}"
        );
    });
}

/// C2: when a required meta-var binding is absent from the instance, the
/// resolver emits a `MissingBinding` diagnostic and skips the instance.
#[test]
fn c2_missing_meta_var_binding_warns_and_skips() {
    with_pattern(PATTERN_DECL, |pattern| {
        let raw = vec![(
            "sync".to_string(),
            vec![instance(
                &["$1"],
                &[
                    // "T" binding is deliberately absent.
                    ("U", "std::sync::MutexGuard<$1>"),
                    ("lock", "$U::lock"),
                    ("unlock", "$U::drop"),
                ],
            )],
        )];
        let (cfg, diags) = resolve_ops_config(pattern, &raw);

        assert!(
            matches!(&diags[..], [ResolveDiagnostic::MissingBinding { .. }]),
            "expected exactly one MissingBinding diagnostic, got: {diags:?}"
        );
        assert!(
            cfg.instances_of("sync").is_empty(),
            "instance with missing T must be skipped"
        );
    });
}

/// C4: when the TOML instance contains a key that is not a declared meta-var
/// or op name, the resolver emits an `UnknownKey` diagnostic and skips it.
#[test]
fn c4_unknown_extra_key_warns_and_skips() {
    with_pattern(PATTERN_DECL, |pattern| {
        let raw = vec![(
            "sync".to_string(),
            vec![instance(
                &["$1"],
                &[
                    ("T", "std::sync::Mutex<$1>"),
                    ("U", "std::sync::MutexGuard<$1>"),
                    ("lock", "$T::lock"),
                    ("unlock", "$U::drop"),
                    ("foo", "<bogus>"), // not a declared meta-var or op name
                ],
            )],
        )];
        let (cfg, diags) = resolve_ops_config(pattern, &raw);

        assert!(
            diags
                .iter()
                .any(|d| matches!(d, ResolveDiagnostic::UnknownKey { name, .. } if name == "foo")),
            "expected an UnknownKey diagnostic for 'foo', got: {diags:?}"
        );
        assert!(
            cfg.instances_of("sync").is_empty(),
            "instance with unknown key must be skipped"
        );
    });
}

/// C7: when meta-var bindings are mutually recursive (`T = $U`, `U = $T`),
/// the fixed-point expansion does not converge, and a `Cycle` diagnostic is
/// emitted; the instance is skipped.
#[test]
fn c7_cycle_warns_and_skips() {
    with_pattern(PATTERN_DECL, |pattern| {
        let raw = vec![(
            "sync".to_string(),
            vec![instance(
                &["$1"],
                &[
                    ("T", "$U"), // T → $U, but U → $T → forms a cycle
                    ("U", "$T"),
                    ("lock", "$T::lock"),
                    ("unlock", "$U::drop"),
                ],
            )],
        )];
        let (cfg, diags) = resolve_ops_config(pattern, &raw);

        assert!(
            diags.iter().any(|d| matches!(d, ResolveDiagnostic::Cycle { .. })),
            "expected a Cycle diagnostic, got: {diags:?}"
        );
        assert!(cfg.instances_of("sync").is_empty(), "cyclic instance must be skipped");
    });
}

/// Mixed scenario: a good instance and a bad instance (missing `U`) for the
/// same group.  The resolver keeps the good one and skips the bad one.
#[test]
fn good_and_bad_instance_separated() {
    with_pattern(PATTERN_DECL, |pattern| {
        let raw = vec![(
            "sync".to_string(),
            vec![
                // Good: all required bindings present.
                instance(
                    &["$1"],
                    &[
                        ("T", "std::sync::Mutex<$1>"),
                        ("U", "std::sync::MutexGuard<$1>"),
                        ("lock", "$T::lock"),
                        ("unlock", "$U::drop"),
                    ],
                ),
                // Bad: missing "U" binding → should be skipped.
                instance(
                    &["$1"],
                    &[
                        ("T", "parking_lot::Mutex<$1>"),
                        ("lock", "$T::lock"),
                        ("unlock", "$T::drop"),
                    ],
                ),
            ],
        )];
        let (cfg, diags) = resolve_ops_config(pattern, &raw);

        assert_eq!(
            diags.len(),
            1,
            "expected exactly 1 diagnostic (for the bad instance), got: {diags:?}"
        );
        assert_eq!(
            cfg.instances_of("sync").len(),
            1,
            "good instance should be retained, bad one skipped"
        );
    });
}
