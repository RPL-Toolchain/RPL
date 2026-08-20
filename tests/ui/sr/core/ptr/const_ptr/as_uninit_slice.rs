//@compile-flags: -Z mir-opt-level=1 -Z inline-mir=false

#![feature(ptr_as_uninit)]

use std::ptr::{self, NonNull};

unsafe fn unknown_len(ptr: *const [u16]) {
    let _ = unsafe { ptr.as_uninit_slice() };
}

fn main() {
    let data = NonNull::<u16>::dangling().as_ptr();

    unsafe {
        let too_large = ptr::slice_from_raw_parts(data, 4_611_686_018_427_387_904usize);
        let _ = too_large.as_uninit_slice();
        //~^ unsafe_numeric_precondition

        let boundary = ptr::slice_from_raw_parts(data, 4_611_686_018_427_387_903usize);
        let _ = boundary.as_uninit_slice();

        let ordinary = ptr::slice_from_raw_parts(data, 4usize);
        let _ = ordinary.as_uninit_slice();

        let null = ptr::slice_from_raw_parts(ptr::null::<u16>(), usize::MAX);
        let _ = null.as_uninit_slice();

        unknown_len(ordinary);
    }
}
