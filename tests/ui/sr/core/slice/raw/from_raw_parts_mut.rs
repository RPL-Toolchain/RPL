//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

use std::ptr::NonNull;

unsafe fn unknown_len(ptr: *mut [u8; 2], len: usize) {
    let _ = unsafe { std::slice::from_raw_parts_mut(ptr, len) };
}

fn main() {
    unsafe {
        let slice_mut_ptr = std::ptr::null_mut::<u8>();
        let _ = std::slice::from_raw_parts_mut(slice_mut_ptr, 1);
        //~^ unsafe_null_precondition

        let mut values = [1u8, 2, 3];
        let len = values.len();
        let _ = std::slice::from_raw_parts_mut(values.as_mut_ptr(), len);

        let ptr = NonNull::<[u8; 2]>::dangling().as_ptr();

        let overflow = 4_611_686_018_427_387_904usize;
        let _ = std::slice::from_raw_parts_mut(ptr, overflow);
        //~^ unsafe_numeric_precondition

        let boundary = 4_611_686_018_427_387_903usize;
        let _ = std::slice::from_raw_parts_mut(ptr, boundary);

        let ordinary = 1usize;
        let _ = std::slice::from_raw_parts_mut(ptr, ordinary);

        unknown_len(ptr, ordinary);
    }
}
