//! Well-formedness checks for `ops { ... }` blocks (R1–R3).
//!
//! These tests exercise `rpl_context::pat::check_ops_block` — the pre-lowering
//! validator that guards the `unreachable!()` contracts inside `OpsMetaLookup`.

#![allow(internal_features)]
#![feature(rustc_private)]
#![feature(rustc_attrs)]
#![feature(let_chains)]
#![feature(box_patterns)]
#![feature(debug_closure_helpers)]
#![recursion_limit = "256"]

extern crate rustc_data_structures;
extern crate rustc_span;

use rpl_context::pat::check_ops_block;

mod common;
use common::make_static_mctx;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse `src` as an RPL pattern file, extract every `opsBlock` in it, run
/// `check_ops_block` on each one, and return all errors as a flat `Vec<String>`
/// (the display form of each `OpsWfError`).
///
/// Panics if the source fails to parse (that would be a test-fixture bug, not
/// an R1/R2/R3 violation).
fn collect_wf_errors(src: &str) -> Vec<String> {
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

/// Assert that at least one error in `errors` contains `needle`.
#[track_caller]
fn assert_any_contains(errors: &[String], needle: &str) {
    assert!(
        errors.iter().any(|e| e.contains(needle)),
        "expected an error containing {:?} but got: {:?}",
        needle,
        errors
    );
}

// ---------------------------------------------------------------------------
// R1: op-level meta-vars must be of kind `type`
// ---------------------------------------------------------------------------

/// R1: a `const(usize)`-kind meta-var in an op group should be rejected.
#[test]
fn r1_non_type_op_meta_var_const_is_rejected() {
    let src = r#"
pattern test
ops {
    sync[$N: const(usize)] = { fn $lock() -> _; }
}
patt { p[] = fn _ () -> _ {} }
"#;
    let errors = collect_wf_errors(src);
    assert!(!errors.is_empty(), "R1: expected at least one error, got none");
    assert_any_contains(&errors, "must be of kind 'type'");
}

/// R1: a `place`-kind meta-var in an op group should also be rejected.
#[test]
fn r1_non_type_op_meta_var_place_is_rejected() {
    let src = r#"
pattern test
ops {
    sync[$P: place(&i32)] = { fn $lock() -> _; }
}
patt { p[] = fn _ () -> _ {} }
"#;
    let errors = collect_wf_errors(src);
    assert!(!errors.is_empty(), "R1: expected at least one error, got none");
    assert_any_contains(&errors, "must be of kind 'type'");
}

/// R1: a `type`-kind meta-var should be accepted (no error).
#[test]
fn r1_type_kind_op_meta_var_is_accepted() {
    let src = r#"
pattern test
ops {
    sync[$T: type] = { fn $lock(&mut $T) -> _; }
}
patt { p[] = fn _ () -> _ {} }
"#;
    let errors = collect_wf_errors(src);
    let r1_errors: Vec<_> = errors.iter().filter(|e| e.contains("must be of kind")).collect();
    assert!(
        r1_errors.is_empty(),
        "R1: no error expected for type-kind meta-var, got: {r1_errors:?}"
    );
}

// ---------------------------------------------------------------------------
// R2: op signatures may only reference meta-vars declared in the same group
// ---------------------------------------------------------------------------

/// R2: `$Z` is used in the signature but not declared in the op group.
#[test]
fn r2_op_signature_uses_undeclared_meta_var() {
    let src = r#"
pattern test
ops {
    sync[$T: type] = { fn $lock(&mut $Z) -> _; }
}
patt { p[] = fn _ () -> _ {} }
"#;
    let errors = collect_wf_errors(src);
    assert!(!errors.is_empty(), "R2: expected at least one error, got none");
    assert_any_contains(&errors, "'$Z' is not declared in op group 'sync'");
}

/// R2: all meta-vars referenced are declared — no error expected.
#[test]
fn r2_all_referenced_meta_vars_declared_is_accepted() {
    let src = r#"
pattern test
ops {
    sync[$T: type, $U: type] = {
        fn $lock(&mut $T) -> $U;
        fn $unlock(&mut $U) -> _;
    }
}
patt { p[] = fn _ () -> _ {} }
"#;
    let errors = collect_wf_errors(src);
    let r2_errors: Vec<_> = errors
        .iter()
        .filter(|e| e.contains("is not declared in op group"))
        .collect();
    assert!(r2_errors.is_empty(), "R2: no error expected, got: {r2_errors:?}");
}

/// R2: the error message includes both the undeclared var name and the group name.
#[test]
fn r2_error_message_names_group_and_var() {
    let src = r#"
pattern test
ops {
    alloc[$T: type] = { fn $allocate(&mut $MISSING) -> _; }
}
patt { p[] = fn _ () -> _ {} }
"#;
    let errors = collect_wf_errors(src);
    assert_any_contains(&errors, "'$MISSING' is not declared in op group 'alloc'");
}

// ---------------------------------------------------------------------------
// R3: op signatures must not contain concrete Rust types
// ---------------------------------------------------------------------------

/// R3: a `TypePath` like `std::sync::Mutex<i32>` in a parameter is rejected.
#[test]
fn r3_op_signature_with_concrete_type_path_is_rejected() {
    let src = r#"
pattern test
ops {
    sync[$T: type] = { fn $lock(&mut std::sync::Mutex<$T>) -> _; }
}
patt { p[] = fn _ () -> _ {} }
"#;
    let errors = collect_wf_errors(src);
    assert!(!errors.is_empty(), "R3: expected at least one error, got none");
    assert_any_contains(&errors, "concrete types belong in rpl.toml");
}

/// R3: a primitive type like `i32` in a parameter is rejected.
#[test]
fn r3_op_signature_with_primitive_type_is_rejected() {
    let src = r#"
pattern test
ops {
    sync[$T: type] = { fn $lock(i32) -> _; }
}
patt { p[] = fn _ () -> _ {} }
"#;
    let errors = collect_wf_errors(src);
    assert!(!errors.is_empty(), "R3: expected at least one error, got none");
    assert_any_contains(&errors, "concrete types belong in rpl.toml");
}

/// R3: `&mut $T` is NOT a concrete type — only the leaf `$T` matters, which is
/// a meta-var.  No error expected.
#[test]
fn r3_ref_to_meta_var_is_accepted() {
    let src = r#"
pattern test
ops {
    sync[$T: type] = { fn $lock(&mut $T) -> _; }
}
patt { p[] = fn _ () -> _ {} }
"#;
    let errors = collect_wf_errors(src);
    let r3_errors: Vec<_> = errors.iter().filter(|e| e.contains("concrete types belong")).collect();
    assert!(
        r3_errors.is_empty(),
        "R3: no error expected for &mut $T, got: {r3_errors:?}"
    );
}

/// R3: concrete type in the return position is also rejected.
#[test]
fn r3_concrete_return_type_is_rejected() {
    let src = r#"
pattern test
ops {
    sync[$T: type] = { fn $lock(&mut $T) -> i32; }
}
patt { p[] = fn _ () -> _ {} }
"#;
    let errors = collect_wf_errors(src);
    assert!(
        !errors.is_empty(),
        "R3: expected at least one error for concrete return type, got none"
    );
    assert_any_contains(&errors, "concrete types belong in rpl.toml");
}

// ---------------------------------------------------------------------------
// Valid input: all rules satisfied, no errors
// ---------------------------------------------------------------------------

/// Sanity check: a well-formed ops block with two type meta-vars produces no errors.
#[test]
fn valid_ops_block_produces_no_errors() {
    let src = r#"
pattern test
ops {
    sync[$T: type, $U: type] = {
        fn $lock(&mut $T) -> $U;
        fn $unlock(&mut $U) -> _;
    }
}
patt { p[] = fn _ () -> _ {} }
"#;
    let errors = collect_wf_errors(src);
    assert!(
        errors.is_empty(),
        "expected no errors for valid ops block, got: {errors:?}"
    );
}

/// Sanity check: ops block with no meta-vars and no params produces no errors.
#[test]
fn ops_block_with_no_meta_vars_produces_no_errors() {
    let src = r#"
pattern test
ops {
    io[] = { fn $read() -> _; }
}
patt { p[] = fn _ () -> _ {} }
"#;
    let errors = collect_wf_errors(src);
    assert!(
        errors.is_empty(),
        "expected no errors for ops block with no meta-vars, got: {errors:?}"
    );
}

// ---------------------------------------------------------------------------
// Integration tests: verify add_parsed_patterns does not panic on bad input
// ---------------------------------------------------------------------------

/// End-to-end guard: feeding a non-type meta-var (`$N: const(usize)`) through
/// the full `add_parsed_patterns` pipeline must not panic.  The R1 check
/// inside `add_ops_block` skips the bad group before any lowering code runs.
#[test]
fn integration_r1_does_not_panic_on_bad_input() {
    use rpl_context::PatternCtxt;

    let src = r#"
pattern test
ops {
    sync[$N: const(usize)] = { fn $lock() -> _; }
}
patt { p[] = fn _ () -> _ {} }
"#;
    let mctx = make_static_mctx("test_r1.rpl", src);

    // This must NOT panic — the R1 guard skips the bad group.
    PatternCtxt::entered_no_tcx(|pcx| {
        pcx.add_parsed_patterns(mctx);
        // The bad group should be absent from ops_block.groups.
        pcx.for_each_rpl_pattern(|_, pattern| {
            assert!(
                pattern.ops_block.groups.is_empty(),
                "expected no groups (bad group was rejected by R1), got: {:?}",
                pattern.ops_block.groups.keys().collect::<Vec<_>>()
            );
        });
    });
}
