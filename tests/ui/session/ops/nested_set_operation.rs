//@rustc-env: RPL_PATS=tests/ui/session/ops/nested_set_operation.rpl
//@compile-flags: -Zinline-mir=false

#![allow(dead_code)]

use std::sync::{Mutex, RwLock};

#[inline(never)]
fn cover<T>(guard: T) {
    drop(guard);
}

#[inline(never)]
fn exempt() {}

// The Mutex assignment survives; the RwLock assignment is subtracted.
// Re-expanding the negative arm across all instances would wrongly suppress it.
fn mutex_uncovered(m: &Mutex<i32>, r: &RwLock<i32>) { //~ ERROR: an abstract lock instance survives nested subtraction
    drop(m.lock());
    cover(r.read());
}

// The opposite arrangement also survives, catching order-dependent caches.
fn rwlock_uncovered(m: &Mutex<i32>, r: &RwLock<i32>) { //~ ERROR: an abstract lock instance survives nested subtraction
    cover(m.lock());
    drop(r.read());
}

// Each assignment's guard flows to cover and there is no exemption:
// both assignments are subtracted.
fn fully_covered(m: &Mutex<i32>, r: &RwLock<i32>) {
    cover(m.lock());
    cover(r.read());
}

// The inner subtraction removes coverage, so the outer positive survives.
fn nested_exemption(m: &Mutex<i32>, r: &RwLock<i32>) { //~ ERROR: an abstract lock instance survives nested subtraction
    cover(m.lock());
    cover(r.read());
    exempt();
}

fn main() {}
