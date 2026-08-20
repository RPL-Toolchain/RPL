//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

#![feature(slice_ptr_get)]

use std::ops::Range;

unsafe fn unknown_index(ptr: *mut [u8], index: usize, range: Range<usize>) {
    let _ = unsafe { ptr.get_unchecked_mut(index) };
    let _ = unsafe { ptr.get_unchecked_mut(range) };
}

fn main() {
    let mut values = [1u8, 2, 3];
    let ptr = &mut values as *mut [u8];

    unsafe {
        let index_out_of_bounds = 3usize;
        let _ = ptr.get_unchecked_mut(index_out_of_bounds);
        //~^ unsafe_numeric_precondition

        let start_after_end = 2usize..1usize;
        let _ = ptr.get_unchecked_mut(start_after_end);
        //~^ unsafe_numeric_precondition

        let end_out_of_bounds = 0usize..4usize;
        let _ = ptr.get_unchecked_mut(end_out_of_bounds);
        //~^ unsafe_numeric_precondition

        let valid_index = 2usize;
        let _ = ptr.get_unchecked_mut(valid_index);
        let valid_range = 0usize..3usize;
        let _ = ptr.get_unchecked_mut(valid_range);

        unknown_index(ptr, valid_index, 0usize..3usize);
    }
}
