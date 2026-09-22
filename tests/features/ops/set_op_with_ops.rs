//@rustc-env: RPL_PATS=tests/features/ops/set_op_with_ops.rpl
//@compile-flags: -Zinline-mir=false
#![feature(core_intrinsics)]
#![allow(internal_features)]
use std::sync::Mutex;

// Locks without explicit "mark" — should match (p_lock matches, p_uncovered doesn't).
fn lock_only() {
    let m: Mutex<i32> = Mutex::new(0);
    let _g = m.lock();
    //~^ ERROR: lock without explicit release
}

// Locks and then calls black_box on the guard (the "covered" marker).
// p_uncovered matches because both the lock call AND the black_box call are present.
// p_lock - p_uncovered subtracts this match → no diagnostic.
//
// Uses `std::intrinsics::black_box` to align with the path bound in
// `rpl.toml`'s [[ops.sync_so]] `unlock = "std::intrinsics::black_box"`.
// The `std::hint::black_box` re-export resolves to a different DefId at MIR
// level, so the matcher would not recognise it (and the test would fail
// silently if MIR inlining ever changed devirtualisation behaviour).
fn lock_and_mark() {
    let m: Mutex<i32> = Mutex::new(0);
    let g = m.lock();
    let _r = std::intrinsics::black_box(g);
}

fn main() {
    lock_only();
    lock_and_mark();
}
