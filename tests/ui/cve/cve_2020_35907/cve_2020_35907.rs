//@revisions: inline regular inline100
//@[inline] compile-flags: -Z inline-mir=true
//@[inline100] compile-flags: -Z inline-mir=true -Z inline-mir-threshold=100
//@[regular] compile-flags: -Z inline-mir=false

use std::cell::UnsafeCell;
use std::task::{RawWaker, RawWakerVTable, Waker};

fn noop_waker() -> Waker {
    unsafe fn clone(_data: *const ()) -> RawWaker {
        RawWaker::new(std::ptr::null(), &NOOP_WAKER_VTABLE)
    }

    unsafe fn wake(_data: *const ()) {}

    unsafe fn wake_by_ref(_data: *const ()) {}

    unsafe fn drop(_data: *const ()) {}

    static NOOP_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

    unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &NOOP_WAKER_VTABLE)) }
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
pub fn noop_waker_ref() -> &'static Waker {
    //~^ERROR: it is unsound to expose a `&'static std::task::Waker` from a thread-local where `std::task::Waker` is `Sync`
    thread_local! {
        static NOOP_WAKER_INSTANCE: UnsafeCell<Waker> =
            UnsafeCell::new(noop_waker());
    }
    NOOP_WAKER_INSTANCE.with(|l| unsafe { &*l.get() })
}

pub fn static_ref() -> &'static i32 {
    thread_local! {
        static THREAD_LOCAL: UnsafeCell<i32> = UnsafeCell::new(0);
    }
    // OK because it is not a return value
    let _ret = THREAD_LOCAL.with(|l| unsafe { &*l.get() });
    &0
}

fn main() {}
