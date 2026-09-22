//! End-to-end lowering test: `ops { ... }` block populates `Pattern.ops_block`.
//!
//! Uses the full parse → collect → lower pipeline so we exercise the same code
//! path as the real driver.  `Box::leak` is used to obtain `'static` arenas
//! that satisfy the lifetime requirements of `PatternCtxt::entered_no_tcx`.

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
use rustc_span::Symbol;

mod common;
use common::make_static_mctx;

/// Full-pipeline test: parse an RPL source string containing an `ops { ... }`
/// block and verify that the resulting `Pattern.ops_block` is populated with
/// the expected groups and operation names.
#[test]
fn lowering_populates_ops_block() {
    let src = r#"
pattern test
ops {
    sync[$T: type, $U: type] = {
        fn $lock(&mut $T) -> $U;
        fn $unlock(&mut $U) -> _;
    }
}
patt {
    p[] = fn _ (..) -> _ { let $x: usize = _; }
}
"#;

    let mctx = make_static_mctx("test.rpl", src);

    PatternCtxt::entered_no_tcx(|pcx| {
        pcx.add_parsed_patterns(mctx);

        let mut found_sync = false;
        pcx.for_each_rpl_pattern(|_, pattern| {
            let groups = &pattern.ops_block.groups;
            if groups.is_empty() {
                return;
            }

            // The `sync` group must be present.
            let sync_group = groups
                .get(&Symbol::intern("sync"))
                .expect("sync op group should be lowered into ops_block.groups");

            assert_eq!(sync_group.ops.len(), 2, "sync group should have exactly 2 ops");

            assert!(
                sync_group.ops.contains_key(&Symbol::intern("lock")),
                "sync group should contain `lock` op (bare, no $ prefix)"
            );
            assert!(
                sync_group.ops.contains_key(&Symbol::intern("unlock")),
                "sync group should contain `unlock` op (bare, no $ prefix)"
            );

            // Verify meta_vars: $T and $U are both type variables.
            assert_eq!(
                sync_group.meta_vars.ty_vars.len(),
                2,
                "sync group should have 2 type meta-variables ($T, $U)"
            );

            found_sync = true;
        });

        assert!(found_sync, "No pattern with a sync ops group was found");
    });
}

/// Verify that two op groups within a single `ops { ... }` block are both
/// lowered into `Pattern.ops_block.groups`.
#[test]
fn lowering_multiple_op_groups_in_one_block() {
    let src = r#"
pattern test
ops {
    sync[$T: type, $U: type] = {
        fn $lock(&mut $T) -> $U;
    }
    alloc[$T: type] = {
        fn $allocate(&mut $T) -> _;
    }
}
patt {
    p[] = fn _ (..) -> _ { let $x: usize = _; }
}
"#;

    let mctx = make_static_mctx("test_multi_groups.rpl", src);

    PatternCtxt::entered_no_tcx(|pcx| {
        pcx.add_parsed_patterns(mctx);

        let mut found = false;
        pcx.for_each_rpl_pattern(|_, pattern| {
            let groups = &pattern.ops_block.groups;
            if groups.is_empty() {
                return;
            }

            assert_eq!(groups.len(), 2, "expected exactly 2 op groups");

            let sync_group = groups
                .get(&Symbol::intern("sync"))
                .expect("sync op group should be present");
            assert_eq!(sync_group.ops.len(), 1, "sync group should have 1 op");
            assert!(
                sync_group.ops.contains_key(&Symbol::intern("lock")),
                "sync group should contain `lock` op"
            );
            assert_eq!(
                sync_group.meta_vars.ty_vars.len(),
                2,
                "sync group should have 2 type meta-variables ($T, $U)"
            );

            let alloc_group = groups
                .get(&Symbol::intern("alloc"))
                .expect("alloc op group should be present");
            assert_eq!(alloc_group.ops.len(), 1, "alloc group should have 1 op");
            assert!(
                alloc_group.ops.contains_key(&Symbol::intern("allocate")),
                "alloc group should contain `allocate` op"
            );
            assert_eq!(
                alloc_group.meta_vars.ty_vars.len(),
                1,
                "alloc group should have 1 type meta-variable ($T)"
            );

            found = true;
        });

        assert!(found, "No pattern with multiple ops groups was found");
    });
}

/// Verify that a pattern file with no `ops` block yields an empty
/// `ops_block.groups` map (not a crash or poison entry).
#[test]
fn no_ops_block_yields_empty_ops_block() {
    let src = r#"
pattern test
patt {
    p[] = fn _ (..) -> _ { let $x: usize = _; }
}
"#;

    let mctx = make_static_mctx("test_no_ops.rpl", src);

    PatternCtxt::entered_no_tcx(|pcx| {
        pcx.add_parsed_patterns(mctx);
        pcx.for_each_rpl_pattern(|_, pattern| {
            assert!(
                pattern.ops_block.groups.is_empty(),
                "pattern without ops block should have empty ops_block.groups"
            );
        });
    });
}
