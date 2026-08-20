//@rustc-env: RPL_PATS=docs/patterns-safedrop/double_free.rpl
//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=true
use std::alloc::{Layout, alloc, dealloc};

#[inline(never)]
unsafe fn release(ptr: *mut u8, layout: Layout) {
    unsafe {
        dealloc(ptr, layout);
    }
}

#[inline(never)]
unsafe fn destroy(ptr: *mut Box<u8>) {
    unsafe {
        core::ptr::drop_in_place(ptr);
    }
}

fn main() {
    unsafe {
        let layout = Layout::new::<u8>();
        let ptr = alloc(layout);
        dealloc(ptr, layout);
        dealloc(ptr, layout);
        //~^ safedrop_double_free

        let release_ptr = alloc(layout);
        release(release_ptr, layout);
        release(release_ptr, layout);
        //~^ safedrop_double_free

        {
            let mut boxed = Box::new(2u8);
            let boxed_ptr = &mut boxed as *mut Box<u8>;
            destroy(boxed_ptr);
        }
        //~^ safedrop_double_free
    }
}
