//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

#![feature(slice_ptr_get)]

use std::ops::Range;

unsafe fn unknown_index(ptr: *const [u8], index: usize, range: Range<usize>) {
    let _ = unsafe { ptr.get_unchecked(index) };
    let _ = unsafe { ptr.get_unchecked(range) };
}

fn main() {
    let values = [1u8, 2, 3];
    let ptr = &values as *const [u8];

    unsafe {
        let index_out_of_bounds = 3usize;
        let _ = ptr.get_unchecked(index_out_of_bounds);
        //~^ unsafe_numeric_precondition

        let start_after_end = 2usize..1usize;
        let _ = ptr.get_unchecked(start_after_end);
        //~^ unsafe_numeric_precondition

        let end_out_of_bounds = 0usize..4usize;
        let _ = ptr.get_unchecked(end_out_of_bounds);
        //~^ unsafe_numeric_precondition

        let valid_index = 2usize;
        let _ = ptr.get_unchecked(valid_index);
        let valid_range = 0usize..3usize;
        let _ = ptr.get_unchecked(valid_range);

        unknown_index(ptr, valid_index, 0usize..3usize);
    }
}
