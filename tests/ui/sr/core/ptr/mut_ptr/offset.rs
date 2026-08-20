//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

use std::ptr::NonNull;

unsafe fn unknown_count(ptr: *mut [u8; 2], count: isize) {
    let _ = unsafe { ptr.offset(count) };
}

fn main() {
    let ptr = NonNull::<[u8; 2]>::dangling().as_ptr();

    unsafe {
        let positive_overflow = 9_223_372_036_854_775_807isize;
        let _ = ptr.offset(positive_overflow);
        //~^ unsafe_numeric_precondition

        let negative_overflow = -9_223_372_036_854_775_808isize;
        let _ = ptr.offset(negative_overflow);
        //~^ unsafe_numeric_precondition

        let positive_boundary = 4_611_686_018_427_387_903isize;
        let _ = ptr.offset(positive_boundary);

        let negative_boundary = -4_611_686_018_427_387_904isize;
        let _ = ptr.offset(negative_boundary);

        let ordinary = 1isize;
        let _ = ptr.offset(ordinary);

        unknown_count(ptr, ordinary);
    }
}
