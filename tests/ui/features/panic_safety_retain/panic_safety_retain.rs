//@rustc-env: RPL_PATS=tests/ui/features/panic_safety_retain/panic_safety_retain.rpl
//@compile-flags: -Z inline-mir=false

//! Custom motif wider than Rudra UD: Rudra skips `Vec::set_len(0)` (leak = safe).
//! This file keeps the older retain-like shape under a dedicated `.rpl`.

#![allow(dead_code)]

#[inline(never)]
fn take_next<I: Iterator<Item = u8>>(iter: &mut I) -> Option<u8> {
    iter.next()
}

/// Temporarily shorten a buffer, then call user code.
fn retain_like_tp<I: Iterator<Item = u8>>(buf: &mut Vec<u8>, iter: &mut I) {
    let len = buf.len();
    unsafe {
        buf.set_len(0);
    }
    let _ = take_next(iter);
    //~^ ERROR: length poisoned before a potentially panicking call
    unsafe {
        buf.set_len(len);
    }
}

fn retain_like_tn<I: Iterator<Item = u8>>(buf: &mut Vec<u8>, iter: &mut I) {
    let _ = take_next(iter);
    let _ = buf.len();
}

fn main() {
    let mut v = vec![1u8, 2, 3];
    let mut it = [9u8].into_iter();
    retain_like_tp(&mut v, &mut it);
    retain_like_tn(&mut v, &mut it);
}
