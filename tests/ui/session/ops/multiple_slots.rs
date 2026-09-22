//@rustc-env: RPL_PATS=tests/ui/session/ops/multiple_slots.rpl
//@compile-flags: -Zinline-mir=false

#![allow(dead_code)]

use std::sync::{Mutex, RwLock};

// Each pair fills both required function slots under one group assignment.
// Both assignments must survive the candidate caches and slot deduplication.
fn mutex_first(m: &Mutex<i32>) { //~ ERROR: two functions use the same abstract lock instance
    drop(m.lock());
}

fn mutex_second(m: &Mutex<i32>) { //~ ERROR: two functions use the same abstract lock instance
    drop(m.lock());
}

fn rwlock_first(r: &RwLock<i32>) { //~ ERROR: two functions use the same abstract lock instance
    drop(r.read());
}

fn rwlock_second(r: &RwLock<i32>) { //~ ERROR: two functions use the same abstract lock instance
    drop(r.read());
}

fn main() {}
