//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

use std::alloc::{GlobalAlloc, Layout, System};
use std::ptr::NonNull;

unsafe fn unknown_size(ptr: *mut u8, layout: Layout, new_size: usize) {
    let _ = unsafe { GlobalAlloc::realloc(&System, ptr, layout, new_size) };
}

fn main() {
    let ptr = NonNull::<u8>::dangling().as_ptr();
    let layout = Layout::new::<u8>();

    unsafe {
        let zero_size = 0usize;
        let _ = GlobalAlloc::realloc(&System, ptr, layout, zero_size);
        //~^ unsafe_numeric_precondition

        let valid_size = 8usize;
        let _ = GlobalAlloc::realloc(&System, ptr, layout, valid_size);

        unknown_size(ptr, layout, valid_size);
    }
}
