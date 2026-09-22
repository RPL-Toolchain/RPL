//@rustc-env: RPL_PATS=tests/features/ops/lock_unlock.rpl
//@compile-flags: -Zinline-mir=false
use std::sync::Mutex;

fn main() {
    let m: Mutex<i32> = Mutex::new(0);
    let _g = m.lock().unwrap();
    //~^ ERROR: lock-unlock pair detected via abstract op
}
