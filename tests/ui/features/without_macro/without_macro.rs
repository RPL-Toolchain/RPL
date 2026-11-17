//@rustc-env: RPL_PATS=tests/ui/features/without_macro/without_macro.rpl
//@compile-flags: -Z inline-mir=false

//! `without!` macro sugar: desugars to `{ set_len(0) } - { set_len(0); set_len(1) }`.
//! `set_len_zero_only` fires; `set_len_zero_then_one` is subtracted.

#![allow(dead_code)]

fn set_len_zero_only<T>(v: &mut Vec<T>) {
    unsafe {
        v.set_len(0);
        //~^ ERROR: `Vec::set_len(0)` without a following `set_len(1)`
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
