//@revisions: inline normal
//@compile-flags: -A rpl::all
//@[inline]compile-flags: -Z inline-mir=true
//@[normal]compile-flags: -Z inline-mir=false
//@check-pass

use std::cell::UnsafeCell;
use std::mem::{ManuallyDrop, transmute};

#[rpl::dynamic(primary_message = "This is a warning")]
fn a() {}

fn b() {
    unsafe {
        let mut a = ManuallyDrop::new("1".to_owned());
        ManuallyDrop::drop(&mut a);
        ManuallyDrop::drop(&mut a);
    }
}

fn c() -> &'static u8 {
    thread_local! {
        static VALUE: UnsafeCell<u8> = UnsafeCell::new(42);
    }
    // unsafe { transmute(&VALUE) }
    VALUE.with(|l| unsafe { &*l.get() })
}
