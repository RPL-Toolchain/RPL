//@compile-flags: -Z inline-mir=false

#![allow(dead_code)]

use std::ptr;

/// TP: strong bypass (`copy_nonoverlapping`) then `drop_in_place` (generic_drop sink).
fn copy_then_drop<T>(src: *const T, dst: *mut T) {
    unsafe {
        ptr::copy_nonoverlapping(src, dst, 1);
        ptr::drop_in_place(dst);
        //~^ ERROR: lifetime-bypassing operation reaches potentially panicking / unresolvable generic code
    }
}

/// TP: strong bypass then unresolvable `FnOnce`.
fn read_then_callback<T, F: FnOnce()>(p: *const T, f: F) {
    unsafe {
        let _v = ptr::read(p);
        f();
        //~^ ERROR: lifetime-bypassing operation reaches potentially panicking / unresolvable generic code
    }
}

/// TN: `set_len(0)` is skipped (Rudra: leaking is safe).
fn set_len_zero_then_cb<F: FnOnce()>(buf: &mut Vec<u8>, f: F) {
    unsafe {
        buf.set_len(0);
    }
    f();
}

/// TN: `ptr::write` on `Copy` is skipped.
fn write_copy_then_cb<F: FnOnce()>(p: *mut u8, f: F) {
    unsafe {
        ptr::write(p, 1u8);
    }
    f();
}

/// TP: intervening statements between bypass and sink still CFG-reach.
fn read_then_noise_then_callback<T, F: FnOnce()>(p: *const T, f: F) {
    unsafe {
        let _v = ptr::read(p);
        let _n = 1usize + 1;
        f();
        //~^ ERROR: lifetime-bypassing operation reaches potentially panicking / unresolvable generic code
    }
}

/// TN: sink before bypass — no CFG reachability from bypass to sink.
fn callback_then_copy<T, F: FnOnce()>(src: *const T, dst: *mut T, f: F) {
    f();
    unsafe {
        ptr::copy_nonoverlapping(src, dst, 1);
    }
}

fn main() {
    let mut x = String::from("a");
    let mut y = String::from("b");
    copy_then_drop(&x, &mut y);
    read_then_callback(&x, || ());
    read_then_noise_then_callback(&x, || ());
    write_copy_then_cb(&mut 0u8, || ());
    callback_then_copy(&x, &mut y, || ());
    let mut v = vec![1u8, 2];
    set_len_zero_then_cb(&mut v, || ());
}
