//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

use std::ptr::NonNull;

unsafe fn unknown_count(ptr: *mut [u8; 2], count: usize) {
    let _ = unsafe { ptr.add(count) };
}

fn main() {
    let ptr = NonNull::<[u8; 2]>::dangling().as_ptr();

    unsafe {
        let overflow = 4_611_686_018_427_387_904usize;
        let _ = ptr.add(overflow);
        //~^ unsafe_numeric_precondition

        let boundary = 4_611_686_018_427_387_903usize;
        let _ = ptr.add(boundary);

        let ordinary = 1usize;
        let _ = ptr.add(ordinary);

        unknown_count(ptr, ordinary);
    }
}
