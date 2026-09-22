//@rustc-env: RPL_PATS=tests/ui/session/ops/multiple_slots.rpl
//@compile-flags: -Zinline-mir=false
//@check-pass

#![allow(dead_code)]

use std::sync::{Mutex, RwLock};

// There is only one function for each configured instance. Neither assignment
// can fill both required slots: combinations must not be chosen per slot.
fn mutex_only(m: &Mutex<i32>) {
    drop(m.lock());
}

fn rwlock_only(r: &RwLock<i32>) {
    drop(r.read());
}

fn main() {}
