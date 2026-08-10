//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

#![feature(slice_index_methods)]

use std::ops::RangeFrom;
use std::slice::SliceIndex;

unsafe fn unknown_start(ptr: *const [u8], range: RangeFrom<usize>) {
    let _ = unsafe { <RangeFrom<usize> as SliceIndex<[u8]>>::get_unchecked(range, ptr) };
}

fn main() {
    let values = [1u8, 2, 3];
    let ptr = &values as *const [u8];

    unsafe {
        let start_out_of_bounds = 4usize..;
        let _ = <RangeFrom<usize> as SliceIndex<[u8]>>::get_unchecked(start_out_of_bounds, ptr);
        //~^ unsafe_numeric_precondition

        let boundary = 3usize..;
        let _ = <RangeFrom<usize> as SliceIndex<[u8]>>::get_unchecked(boundary, ptr);

        unknown_start(ptr, 1usize..);
    }
}
