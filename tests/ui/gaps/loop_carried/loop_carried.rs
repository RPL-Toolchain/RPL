//@rustc-env: RPL_PATS=tests/ui/gaps/loop_carried/loop_carried.rpl

//! Faithfulness boundary: loops (multiplicity, not a missing edge).
//!
//! Inside the body the read precedes the write, so the only write->read edge is
//! carried by the back edge.  The match below shows that `alpha` *does* represent
//! the loop-carried dependence -- folded onto one static edge.  What it cannot
//! represent is how many iterations carry it, so counting properties are
//! inexpressible; that part has no match/no-match test by construction.
#![allow(dead_code)]
#![allow(unsafe_op_in_unsafe_fn)]

fn lp2(n: u8) -> u8 {
    let mut s = 9u8;
    for i in 0..n {
        let t = s; //~ ERROR: value produced before this iteration wrote it
        s = i;
        let _ = t;
    }
    n
}
