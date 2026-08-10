//@compile-flags: -Z mir-opt-level=1 -Z inline-mir=false
#![feature(ptr_as_uninit)]

use std::ptr::{self, NonNull};

unsafe fn unknown_len(ptr: *mut [u16]) {
    let _ = unsafe { ptr.as_uninit_slice_mut() };
}

fn main() {
    let data = NonNull::<u16>::dangling().as_ptr();

    unsafe {
        let too_large = ptr::slice_from_raw_parts_mut(data, 4_611_686_018_427_387_904usize);
        let _ = too_large.as_uninit_slice_mut();
        //~^ unsafe_numeric_precondition

        let boundary = ptr::slice_from_raw_parts_mut(data, 4_611_686_018_427_387_903usize);
        let _ = boundary.as_uninit_slice_mut();

        let ordinary = ptr::slice_from_raw_parts_mut(data, 8);
        let _ = ordinary.as_uninit_slice_mut();

        let null = ptr::slice_from_raw_parts_mut(ptr::null_mut::<u16>(), usize::MAX);
        let _ = null.as_uninit_slice_mut();

        unknown_len(ordinary);
    }
}
