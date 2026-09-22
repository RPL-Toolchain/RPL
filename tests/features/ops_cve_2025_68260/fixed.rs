//@rustc-env: RPL_PATS=tests/features/ops_cve_2025_68260/pattern.rpl
//@check-pass
//@compile-flags: -Zinline-mir=false
//! CVE-2025-68260 POC — fixed variant.
//!
//! Mirrors the post-fix shape:
//!   1. Acquire lock.
//!   2. Transfer nodes to a temporary list (under the lock).
//!   3. Drain the temporary list UNDER the lock — no race window.
//!   4. Release the lock (guard drops naturally — no explicit unlock sink call).
//!
//! The `ops_cve_2025_68260` pattern requires an explicit $sync_cve::$unlock call
//! (bound to std::intrinsics::black_box) between transfer and drain.  Since this
//! fixed variant lets the guard drop naturally (no black_box call), the pattern
//! cannot match — demonstrating the structural asymmetry.
#![feature(core_intrinsics)]
#![allow(internal_features, dead_code, unused_must_use)]
use std::sync::Mutex;

#[inline(never)]
fn transfer_to_temp<L, T>(_owning: &mut L, _temp: &mut T) {}

#[inline(never)]
fn drain_temp<T>(_temp: &mut T) {}

struct OwningList {
    dummy: i32,
}

struct TempList {
    dummy: i32,
}

fn fixed_release() {
    let mtx: Mutex<OwningList> = Mutex::new(OwningList { dummy: 0 });

    // Step 1: acquire lock.
    let _owning = mtx.lock();

    let mut list_inner = OwningList { dummy: 0 };
    let mut temp = TempList { dummy: 0 };

    // Step 2: transfer nodes to temp list (still under the lock).
    transfer_to_temp(&mut list_inner, &mut temp);

    // Step 3: drain the temp list WHILE THE LOCK IS HELD — no race window.
    drain_temp(&mut temp);

    // Step 4: lock is released here by natural drop of _owning.
    // No explicit black_box call → no $sync_cve::$unlock match → pattern cannot fire.
}

fn main() {
    fixed_release();
}
