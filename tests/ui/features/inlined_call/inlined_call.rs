//@rustc-env: RPL_PATS=tests/ui/features/inlined_call/inlined_call.rpl

//! Verify that a pattern matching a specific function (`Vec::set_len`) also matches when the
//! function is inlined (so the call itself disappears from MIR). `Vec::set_len` is `#[inline]`,
//! and the driver defaults to `-Z inline-mir=true`, so it is inlined here. The `#[inline(never)]`
//! on `set_len_called` keeps the outer call a real call, so the pattern matches only once.

#![allow(dead_code)]

#[inline(never)]
fn set_len_called<T>(v: &mut Vec<T>) {
    unsafe {
        v.set_len(1);
        //~^ ERROR: inlined `Vec::set_len` call detected
    }
}

fn main() {
    let mut v = vec![1u8, 2, 3];
    set_len_called(&mut v);
}
