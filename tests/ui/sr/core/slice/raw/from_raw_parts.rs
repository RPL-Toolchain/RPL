//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

use std::ptr::NonNull;

unsafe fn unknown_len(ptr: *const [u8; 2], len: usize) {
    let _ = unsafe { std::slice::from_raw_parts(ptr, len) };
}

fn main() {
    unsafe {
        let slice_ptr = std::ptr::null::<u8>();
        let _ = std::slice::from_raw_parts(slice_ptr, 1);
        //~^ unsafe_null_precondition

        let values = [1u8, 2, 3];
        let _ = std::slice::from_raw_parts(values.as_ptr(), values.len());

        let ptr = NonNull::<[u8; 2]>::dangling().as_ptr() as *const [u8; 2];

        let overflow = 4_611_686_018_427_387_904usize;
        let _ = std::slice::from_raw_parts(ptr, overflow);
        //~^ unsafe_numeric_precondition

        let boundary = 4_611_686_018_427_387_903usize;
        let _ = std::slice::from_raw_parts(ptr, boundary);

        let ordinary = 1usize;
        let _ = std::slice::from_raw_parts(ptr, ordinary);

        unknown_len(ptr, ordinary);
    }
}
