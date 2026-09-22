//@rustc-env: RPL_PATS=tests/ui/session/ops/multiple_instances.rpl
//@compile-flags: -Zinline-mir=false

#![allow(dead_code)]

use std::sync::{Mutex, RwLock};

// Both configured sync_2g instances must be explored. Caching candidates only
// by function/pattern would reuse the Mutex combination's empty RwLock result.
fn mutex_instance(m: &Mutex<i32>) { //~ ERROR: abstract lock matched for this configured instance
    drop(m.lock());
}

fn rwlock_instance(r: &RwLock<i32>) { //~ ERROR: abstract lock matched for this configured instance
    drop(r.read());
}

// A plain function must not inherit another candidate's successful match.
fn unrelated() {}

fn main() {}
