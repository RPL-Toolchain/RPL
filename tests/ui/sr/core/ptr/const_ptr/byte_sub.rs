//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

use std::ptr::NonNull;

unsafe fn unknown_count(ptr: *const [u8; 2], count: usize) {
    let _ = unsafe { ptr.byte_sub(count) };
}

fn main() {
    let ptr = NonNull::<[u8; 2]>::dangling().as_ptr() as *const [u8; 2];

    unsafe {
        let overflow = 9_223_372_036_854_775_808usize;
        let _ = ptr.byte_sub(overflow);
        //~^ unsafe_numeric_precondition

        let boundary = 9_223_372_036_854_775_807usize;
        let _ = ptr.byte_sub(boundary);

        let ordinary = 1usize;
        let _ = ptr.byte_sub(ordinary);

        unknown_count(ptr, ordinary);
    }
}
