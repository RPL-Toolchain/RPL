//! Resolver well-formedness checks for `ops { ... }` blocks.
//!
//! Tests R1, R2, and R3 — the checks that guard `OpsMetaLookup`'s
//! `unreachable!()` contracts against malformed input.
//!
//! These tests delegate to `rpl_context::pat::check_ops_block`, which is the
//! pre-lowering validator.  The `rpl_resolve` crate itself handles Rust
//! def-path resolution (path → `DefId`); the ops well-formedness rules are
//! pattern-AST concerns that live in `rpl_context`.  This file keeps the
//! specified test locations while delegating to the correct crate.

#![allow(internal_features)]
#![feature(rustc_private)]
#![feature(rustc_attrs)]
#![feature(let_chains)]
#![feature(box_patterns)]
#![feature(debug_closure_helpers)]
#![recursion_limit = "256"]

extern crate rustc_data_structures;
extern crate rustc_span;

use rpl_context::PatternCtxt;
use rpl_context::pat::{check_ops_block, check_r6_patt_vs_ops};

mod common;
use common::{make_static_mctx, make_static_mctx_with};

// ---------------------------------------------------------------------------
// Helper: pre-lowering R1–R3 checks (ops block well-formedness)
// ---------------------------------------------------------------------------

/// Parse `src`, run `check_ops_block` on every `opsBlock` found, and return
/// all error messages as display strings.
fn run_resolver_on_src(src: &str) -> Vec<String> {
    let mctx = make_static_mctx("test.rpl", src);
    let mut errors = Vec::new();
    for syntax_tree in mctx.syntax_trees.iter() {
        let (_, _, ops, _) = rpl_meta::meta::collect_blocks(syntax_tree);
        for ops_block in ops {
            let errs = check_ops_block(ops_block);
            errors.extend(errs.into_iter().map(|e| e.to_string()));
        }
    }
    errors
}

// ---------------------------------------------------------------------------
// Helper: post-lowering R4–R5 checks (op-ref use-site validation)
// ---------------------------------------------------------------------------

/// Parse, lower, and run R4/R5 use-site checks on `src`.
///
/// `add_parsed_patterns` now automatically calls `check_and_populate_op_refs`
/// internally and stores errors in `Pattern::op_ref_errors`.  This helper
/// collects them into a `Vec<String>` for assertion.
///
/// Panics if the source fails to parse (that's a test-fixture bug).
fn run_r4_r5_checks(src: &str) -> Vec<String> {
    let mctx = make_static_mctx("test_r4r5.rpl", src);
    let mut all_errors: Vec<String> = Vec::new();

    PatternCtxt::entered_no_tcx(|pcx| {
        pcx.add_parsed_patterns(mctx);
        pcx.for_each_rpl_pattern(|_, pattern| {
            all_errors.extend(pattern.op_ref_errors().iter().map(|e| e.to_string()));
        });
    });

    all_errors
}

// ---------------------------------------------------------------------------
// Helper: pre-lowering R6 check (op-level meta-var leak into pattern body)
// ---------------------------------------------------------------------------

/// Parse `src`, run `check_r6_patt_vs_ops`, and return all R6 error messages.
///
/// Uses a **non-panicking** error handler so that `NonLocalMetaVariableNotDeclared`
/// errors from `parse_and_collect` do not abort the check.  The R6 function
/// runs at parse-tree level — before any lowering that would panic.
fn run_r6_check(src: &str) -> Vec<String> {
    // Use a collecting (non-panicking) handler so the meta-collection phase
    // does not abort even when undeclared meta-vars are encountered.
    // Undeclared pattern-level meta-vars are exactly what R6 is meant to flag,
    // so we want the collection phase to keep going.
    let mctx = make_static_mctx_with("test_r6.rpl", src, |_err| {});

    let mut errors = Vec::new();
    for syntax_tree in mctx.syntax_trees.iter() {
        let (_, patts, ops, _) = rpl_meta::meta::collect_blocks(syntax_tree);
        let errs = check_r6_patt_vs_ops(&ops, &patts);
        errors.extend(errs.into_iter().map(|e| e.to_string()));
    }
    errors
}

// ---------------------------------------------------------------------------
// R1: op-level meta-vars must be of kind `type`
// ---------------------------------------------------------------------------

#[test]
fn r1_non_type_op_meta_var_is_rejected() {
    let src = r#"
pattern test
ops {
    sync[$N: const(usize)] = { fn $lock() -> _; }
}
patt { p[] = fn _ () -> _ {} }
"#;
    let errs = run_resolver_on_src(src);
    assert!(
        errs.iter().any(|e| e.contains("must be of kind 'type'")),
        "R1: expected an error about non-type op-level meta-var, got: {errs:?}"
    );
}

// ---------------------------------------------------------------------------
// R2: op signatures may only reference meta-vars from the same group
// ---------------------------------------------------------------------------

#[test]
fn r2_op_signature_uses_undeclared_meta_var() {
    let src = r#"
pattern test
ops {
    sync[$T: type] = { fn $lock(&mut $Z) -> _; }
}
patt { p[] = fn _ () -> _ {} }
"#;
    let errs = run_resolver_on_src(src);
    assert!(
        errs.iter()
            .any(|e| e.contains("'$Z' is not declared in op group 'sync'")),
        "R2: expected undeclared-meta-var error, got: {errs:?}"
    );
}

// ---------------------------------------------------------------------------
// R3: op signatures must not contain concrete Rust paths/types
// ---------------------------------------------------------------------------

#[test]
fn r3_op_signature_with_concrete_path_is_rejected() {
    let src = r#"
pattern test
ops {
    sync[$T: type] = { fn $lock(&mut std::sync::Mutex<$T>) -> _; }
}
patt { p[] = fn _ () -> _ {} }
"#;
    let errs = run_resolver_on_src(src);
    assert!(
        errs.iter().any(|e| e.contains("concrete types belong in rpl.toml")),
        "R3: expected concrete-path error, got: {errs:?}"
    );
}

// ---------------------------------------------------------------------------
// R4: $group::$op references must resolve to declared op groups and ops
// ---------------------------------------------------------------------------

#[test]
fn r4_undeclared_op_group_is_rejected() {
    // `$undeclared` is not in the ops block — R4a should fire.
    let src = r#"
pattern test
patt {
    p[$T: type] = fn _ (..) -> _ {
        let $x: $T = $undeclared::$op(_);
    }
}
"#;
    let errs = run_r4_r5_checks(src);
    assert!(
        errs.iter().any(|e| e.contains("op group 'undeclared' is not declared")),
        "R4: expected undeclared-group error, got: {errs:?}"
    );
}

#[test]
fn r4_undeclared_op_in_group_is_rejected() {
    // `$sync` group is declared but has no `$try_lock` — R4b should fire.
    let src = r#"
pattern test
ops { sync[$T: type] = { fn $lock(&mut $T) -> _; } }
patt {
    p[$T: type] = fn _ (..) -> _ {
        let $x: $T = $sync::$try_lock(_);
    }
}
"#;
    let errs = run_r4_r5_checks(src);
    assert!(
        errs.iter()
            .any(|e| e.contains("op 'try_lock' is not declared in op group 'sync'")),
        "R4: expected undeclared-op-in-group error, got: {errs:?}"
    );
}

#[test]
fn r4_valid_op_ref_is_accepted() {
    // A valid `$sync::$lock` should produce no R4 errors.
    let src = r#"
pattern test
ops { sync[$T: type] = { fn $lock(&mut $T) -> _; } }
patt {
    p[$T: type] = fn _ (..) -> _ {
        let $x: $T = $sync::$lock(_);
    }
}
"#;
    let errs = run_r4_r5_checks(src);
    let r4_errs: Vec<_> = errs.iter().filter(|e| e.contains("is not declared")).collect();
    assert!(
        r4_errs.is_empty(),
        "R4: no error expected for valid op ref, got: {r4_errs:?}"
    );
}

// ---------------------------------------------------------------------------
// R5: Op-call arity must match the op signature's arity
// ---------------------------------------------------------------------------

#[test]
fn r5_arity_mismatch_at_call_site() {
    // `$sync::$lock` expects 1 arg (`&mut $T`) but is called with 0.
    let src = r#"
pattern test
ops { sync[$T: type] = { fn $lock(&mut $T) -> _; } }
patt {
    p[$T: type] = fn _ (..) -> _ {
        let $x: $T = $sync::$lock();
    }
}
"#;
    let errs = run_r4_r5_checks(src);
    assert!(
        errs.iter().any(|e| e.contains("arity")),
        "R5: expected arity-mismatch error, got: {errs:?}"
    );
}

#[test]
fn r5_correct_arity_is_accepted() {
    // `$sync::$lock` expects 1 arg and is called with 1.
    let src = r#"
pattern test
ops { sync[$T: type] = { fn $lock(&mut $T) -> _; } }
patt {
    p[$T: type] = fn _ (..) -> _ {
        let $x: $T = $sync::$lock(_);
    }
}
"#;
    let errs = run_r4_r5_checks(src);
    let r5_errs: Vec<_> = errs.iter().filter(|e| e.contains("arity")).collect();
    assert!(r5_errs.is_empty(), "R5: no arity error expected, got: {r5_errs:?}");
}

// ---------------------------------------------------------------------------
// R6: Pattern bodies must not reference op-level meta-vars
// ---------------------------------------------------------------------------

#[test]
fn r6_op_level_meta_var_leaks_into_pattern_body() {
    // `$T` is declared in `sync[$T]` (op-level) but NOT in `p[]`.
    // Using `$T` in the pattern body is an R6 violation.
    let src = r#"
pattern test
ops { sync[$T: type] = { fn $lock(&mut $T) -> _; } }
patt {
    p[] = fn _ (..) -> _ {
        let $x: $T = _;
    }
}
"#;
    let errs = run_r6_check(src);
    assert!(
        errs.iter()
            .any(|e| e.contains("op-level meta-var '$T' cannot appear in a pattern body")),
        "R6: expected op-level-leak error, got: {errs:?}"
    );
}

#[test]
fn r6_pattern_level_meta_var_is_accepted() {
    // `$T` is declared at both pattern level `p[$T]` and op level `sync[$T]`.
    // The pattern body's `$T` refers to the pattern-level one — no R6 error.
    let src = r#"
pattern test
ops { sync[$T: type] = { fn $lock(&mut $T) -> _; } }
patt {
    p[$T: type] = fn _ (..) -> _ {
        let $x: $T = _;
    }
}
"#;
    let errs = run_r6_check(src);
    let r6_errs: Vec<_> = errs.iter().filter(|e| e.contains("op-level meta-var")).collect();
    assert!(
        r6_errs.is_empty(),
        "R6: no error expected when pattern-level $T shadows op-level $T, got: {r6_errs:?}"
    );
}
