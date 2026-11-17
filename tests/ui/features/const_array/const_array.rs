//@rustc-env: RPL_PATS=tests/ui/features/const_array/const_array.rpl

//! A concrete const generic (`[u8; 4]`) in a pattern type must match the target's `[u8; 4]`.

#![allow(dead_code)]

#[inline(never)]
fn foo() {
    let arr = [0u8; 4];
    //~^ ERROR: fixed [u8; 4] array seen
    let _ = arr[0];
}

fn main() {
    foo();
}
