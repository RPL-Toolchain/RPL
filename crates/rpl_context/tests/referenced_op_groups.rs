//! Tests for `RustItems::referenced_op_groups` population.
//!
//! After `add_parsed_patterns` the per-pattern `referenced_op_groups` set
//! must contain exactly the group names that appear as `$group::$op` operands
//! in the pattern's function bodies.

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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Two op groups (`sync` and `logger`) are referenced by OpRefs in the body.
/// The `referenced_op_groups` set must contain exactly those two names.
#[test]
fn referenced_op_groups_collects_distinct_names() {
    let src = r#"
pattern test
ops {
    sync[$T: type] = { fn $lock(&mut $T) -> _; fn $unlock(&mut $T) -> _; }
    logger[$L: type] = { fn $log(&$L) -> _; }
}
patt {
    p[$T: type, $L: type] = fn _ (..) -> _ {
        let $x: $T = $sync::$lock(_);
        let $y: () = $sync::$unlock(_);
        let $z: () = $logger::$log(_);
    }
}
"#;

    let mctx = make_static_mctx("test_refgroups.rpl", src);

    PatternCtxt::entered_no_tcx(|pcx| {
        pcx.add_parsed_patterns(mctx);

        let mut found = false;
        pcx.for_each_rpl_pattern(|_, pattern| {
            let p = pattern
                .patt_block
                .get(&Symbol::intern("p"))
                .expect("pattern item 'p' should exist");
            let groups = p.expect_rust_items().referenced_op_groups();
            assert_eq!(
                groups.len(),
                2,
                "expected 2 referenced op groups, got {}: {:?}",
                groups.len(),
                groups.iter().map(|s| s.as_str()).collect::<Vec<_>>()
            );
            assert!(
                groups.contains(&Symbol::intern("sync")),
                "expected 'sync' in referenced_op_groups"
            );
            assert!(
                groups.contains(&Symbol::intern("logger")),
                "expected 'logger' in referenced_op_groups"
            );
            found = true;
        });

        assert!(found, "no pattern with referenced op groups was found");
    });
}

/// A pattern that references the same group twice should still produce a set
/// of size 1 (deduplication).
#[test]
fn referenced_op_groups_deduplicates() {
    let src = r#"
pattern test
ops {
    sync[$T: type] = { fn $lock(&mut $T) -> _; fn $unlock(&mut $T) -> _; }
}
patt {
    p[$T: type] = fn _ (..) -> _ {
        let $x: $T = $sync::$lock(_);
        let $y: () = $sync::$unlock(_);
    }
}
"#;

    let mctx = make_static_mctx("test_refgroups.rpl", src);

    PatternCtxt::entered_no_tcx(|pcx| {
        pcx.add_parsed_patterns(mctx);

        let mut found = false;
        pcx.for_each_rpl_pattern(|_, pattern| {
            let p = pattern
                .patt_block
                .get(&Symbol::intern("p"))
                .expect("pattern item 'p' should exist");
            let groups = p.expect_rust_items().referenced_op_groups();
            assert_eq!(
                groups.len(),
                1,
                "expected 1 referenced op group (dedup), got {}: {:?}",
                groups.len(),
                groups.iter().map(|s| s.as_str()).collect::<Vec<_>>()
            );
            assert!(
                groups.contains(&Symbol::intern("sync")),
                "expected 'sync' in referenced_op_groups"
            );
            found = true;
        });

        assert!(found, "no pattern was found");
    });
}

/// A pattern with no OpRef calls should have an empty `referenced_op_groups`.
#[test]
fn referenced_op_groups_empty_when_no_op_refs() {
    let src = r#"
pattern test
ops {
    sync[$T: type] = { fn $lock(&mut $T) -> _; }
}
patt {
    p[$T: type] = fn _ (..) -> _ {
        let $x: $T = _;
    }
}
"#;

    let mctx = make_static_mctx("test_refgroups.rpl", src);

    PatternCtxt::entered_no_tcx(|pcx| {
        pcx.add_parsed_patterns(mctx);

        let mut found = false;
        pcx.for_each_rpl_pattern(|_, pattern| {
            let p = pattern
                .patt_block
                .get(&Symbol::intern("p"))
                .expect("pattern item 'p' should exist");
            let groups = p.expect_rust_items().referenced_op_groups();
            assert!(
                groups.is_empty(),
                "expected empty referenced_op_groups, got: {:?}",
                groups.iter().map(|s| s.as_str()).collect::<Vec<_>>()
            );
            found = true;
        });

        assert!(found, "no pattern was found");
    });
}
