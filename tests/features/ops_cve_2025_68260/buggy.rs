//@rustc-env: RPL_PATS=tests/features/ops_cve_2025_68260/pattern.rpl
//@compile-flags: -Zinline-mir=false
//! CVE-2025-68260 POC — buggy variant.
//!
//! Mirrors the pre-fix shape of Linux kernel Node::release:
//!   1. Acquire lock.
//!   2. Transfer nodes to a temporary list (under the lock).
//!   3. Release the lock (black_box as opaque unlock sink).
//!   4. Drain the temporary list OUTSIDE the lock — the race window.
//!
//! The `ops_cve_2025_68260` pattern should fire here.
#![feature(core_intrinsics)]
#![allow(internal_features, dead_code, unused_must_use)]
use std::sync::Mutex;

// Stand-ins for the kernel's intrusive-list operations.
// #[inline(never)] ensures stable, non-inlined MIR call sites even
// when the driver compiles with MIR inlining enabled.
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

fn buggy_release() {
    let mtx: Mutex<OwningList> = Mutex::new(OwningList { dummy: 0 });

    // Step 1: acquire lock — pattern matches $sync_cve::$lock.
    let g = mtx.lock();
    //~^ ERROR: intrusive list drained outside the lock that owns it (CVE-2025-68260 bug class)

    let mut list_inner = OwningList { dummy: 0 };
    let mut temp = TempList { dummy: 0 };

    // Step 2: transfer nodes to temp list under the lock.
    transfer_to_temp(&mut list_inner, &mut temp);

    // Step 3: release the lock — std::intrinsics::black_box as the opaque unlock sink.
    // The guard (g) is moved into the intrinsic, which acts as an opaque consumer.
    std::intrinsics::black_box(g);

    // Step 4: drain the temp list OUTSIDE the lock — the bug window.
    drain_temp(&mut temp);
}

fn main() {
    buggy_release();
}
