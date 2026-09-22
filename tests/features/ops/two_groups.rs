//@rustc-env: RPL_PATS=tests/features/ops/two_groups.rpl
//@compile-flags: -Zinline-mir=false
// two_groups test: exercises cartesian-product matching across TWO op groups.
// sync_2g has 2 instances (Mutex, RwLock).
// logger_2g has 1 instance (std::intrinsics::black_box).
// Total combinations: 2 × 1 = 2.
// Only the (Mutex, black_box) combination fires; (RwLock, black_box) has no RwLock call.
//
// Uses `std::intrinsics::black_box` to align with the path bound in
// `rpl.toml`'s [[ops.logger_2g]] `log = "std::intrinsics::black_box"`.
// See set_op_with_ops.rs for the DefId-mismatch rationale.
#![feature(core_intrinsics)]
#![allow(internal_features)]
use std::sync::Mutex;

fn main() {
    let m: Mutex<i32> = Mutex::new(0);
    let _g = m.lock();
    //~^ ERROR: two-groups match: lock and log in same function
    let msg: &str = "hi";
    let _r = std::intrinsics::black_box(msg);
}
