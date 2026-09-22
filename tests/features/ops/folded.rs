//@rustc-env: RPL_PATS=tests/features/ops/folded.rpl
//@check-pass
//@compile-flags: -Zinline-mir=false
use std::sync::Mutex;

fn main() {
    let m: Mutex<i32> = Mutex::new(0);
    let _g = m.lock().unwrap();
}
