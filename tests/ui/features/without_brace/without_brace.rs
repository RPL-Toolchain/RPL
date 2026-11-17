//@rustc-env: RPL_PATS=tests/ui/features/without_brace/without_brace.rpl
//@compile-flags: -Z inline-mir=false

//! `without!` inside a brace block (`p = { fn ... }`) — the pattern-item form used by
//! `docs/patterns-pest/rudra/panic-safety.rpl`. `set_len_zero_only` fires; the follow-up
//! `set_len(1)` is subtracted.

#![allow(dead_code)]

fn set_len_zero_only<T>(v: &mut Vec<T>) {
    unsafe {
        v.set_len(0);
        //~^ ERROR: `Vec::set_len(0)` without a following `set_len(1)` (brace block)
    }
}

fn set_len_zero_then_one<T>(v: &mut Vec<T>) {
    unsafe {
        v.set_len(0);
        v.set_len(1);
    }
}

fn main() {
    let mut v = vec![1u8, 2, 3];
    set_len_zero_only(&mut v);
    set_len_zero_then_one(&mut v);
}
