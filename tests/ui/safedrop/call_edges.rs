//@rustc-env: RPL_PATS=docs/patterns-safedrop
//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=true
#![allow(dead_code, unsafe_op_in_unsafe_fn)]

use std::alloc::{Layout, alloc, dealloc};

struct CleanupUse {
    ptr: *mut u8,
}

impl Drop for CleanupUse {
    fn drop(&mut self) {
        unsafe { self.ptr.write(1) }
    }
}

#[inline(never)]
unsafe fn free_only_on_cleanup(ptr: *mut u8, layout: Layout, panic_now: bool) {
    if panic_now {
        unsafe { dealloc(ptr, layout) };
        panic!("cleanup");
    }
}

#[inline(never)]
unsafe fn free_only_on_cleanup_wrapper(ptr: *mut u8, layout: Layout, panic_now: bool) {
    unsafe { free_only_on_cleanup(ptr, layout, panic_now) }
}

#[inline(never)]
unsafe fn caller_cleanup(ptr: *mut u8, layout: Layout) {
    let _use_on_cleanup = CleanupUse { ptr };
    unsafe { free_only_on_cleanup_wrapper(ptr, layout, true) };
}
//~^ ERROR: this unsafe operation may free storage that is already dead

fn main() {
    unsafe {
        let layout = Layout::new::<u8>();

        let normal = alloc(layout);
        free_only_on_cleanup_wrapper(normal, layout, false);
        dealloc(normal, layout);

        let cleanup = alloc(layout);
        caller_cleanup(cleanup, layout);
    }
}
