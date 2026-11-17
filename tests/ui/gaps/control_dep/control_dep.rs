//@rustc-env: RPL_PATS=tests/ui/gaps/control_dep/control_dep.rpl

//! Faithfulness boundary: control dependence (false positive).
//!
//! The DDG carries an edge from `x = 1` to the read of `x` in `return x`, but the
//! two branch conditions are complements, so no execution realizes it.  This test
//! records the resulting false positive; once control-dependence edges are paired
//! with branch-condition reasoning, it should turn into a check-pass test.
#![allow(dead_code)]
#![allow(unsafe_op_in_unsafe_fn)]

fn cd(c: bool) -> u8 {
    let mut x = 0u8;
    if c {
        x = 1;
    }
    if !c {
        return x; //~ ERROR: flow from a branch that cannot reach this read
    }
    return 0;
}
