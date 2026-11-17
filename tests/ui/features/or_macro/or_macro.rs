//@rustc-env: RPL_PATS=tests/ui/features/or_macro/or_macro.rpl
//@compile-flags: -Z inline-mir=false

//! `or!` macro sugar: the two `set_len` alternatives are combined with `||`.

#![allow(dead_code)]

fn set_len_zero<T>(v: &mut Vec<T>) {
    unsafe {
        v.set_len(0);
        //~^ ERROR: `Vec::set_len` called with 0 or 1
    }
}

fn set_len_one<T>(v: &mut Vec<T>) {
    unsafe {
        v.set_len(1);
        //~^ ERROR: `Vec::set_len` called with 0 or 1
    }
}

fn set_len_two<T>(v: &mut Vec<T>) {
    unsafe {
        v.set_len(2);
    }
}

fn main() {
    let mut v = vec![1u8, 2, 3];
    set_len_zero(&mut v);
    set_len_one(&mut v);
    set_len_two(&mut v);
}
