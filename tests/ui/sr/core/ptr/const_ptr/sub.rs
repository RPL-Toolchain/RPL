//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

use std::ptr::NonNull;

unsafe fn unknown_count(ptr: *const [u8; 2], count: usize) {
    let _ = unsafe { ptr.sub(count) };
}

fn main() {
    let ptr = NonNull::<[u8; 2]>::dangling().as_ptr() as *const [u8; 2];

    unsafe {
        let overflow = 4_611_686_018_427_387_904usize;
        let _ = ptr.sub(overflow);
        //~^ unsafe_numeric_precondition

        let boundary = 4_611_686_018_427_387_903usize;
        let _ = ptr.sub(boundary);

        let ordinary = 1usize;
        let _ = ptr.sub(ordinary);

        unknown_count(ptr, ordinary);
    }
}
