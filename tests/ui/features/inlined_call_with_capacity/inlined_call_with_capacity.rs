//@rustc-env: RPL_PATS=tests/ui/features/inlined_call_with_capacity/inlined_call_with_capacity.rpl

//! `Vec::with_capacity` is `#[inline]` and its surviving MIR spans several allocation statements:
//! one call yields several matches, so several identical reports share the call-site span.

#![allow(dead_code)]

#[inline(never)]
fn foo() -> Vec<u8> {
    Vec::with_capacity(10)
    //~^ ERROR: inlined `Vec::with_capacity` detected
    //~| ERROR: inlined `Vec::with_capacity` detected
    //~| ERROR: inlined `Vec::with_capacity` detected
}

fn main() {
    let _ = foo();
}
