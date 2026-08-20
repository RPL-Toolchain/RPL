//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

#![feature(slice_index_methods)]

use std::ops::RangeTo;
use std::slice::SliceIndex;

unsafe fn unknown_end(ptr: *const [u8], range: RangeTo<usize>) {
    let _ = unsafe { <RangeTo<usize> as SliceIndex<[u8]>>::get_unchecked(range, ptr) };
}

fn main() {
    let values = [1u8, 2, 3];
    let ptr = &values as *const [u8];

    unsafe {
        let end_out_of_bounds = ..4usize;
        let _ = <RangeTo<usize> as SliceIndex<[u8]>>::get_unchecked(end_out_of_bounds, ptr);
        //~^ unsafe_numeric_precondition

        let boundary = ..3usize;
        let _ = <RangeTo<usize> as SliceIndex<[u8]>>::get_unchecked(boundary, ptr);

        unknown_end(ptr, ..2usize);
    }
}
