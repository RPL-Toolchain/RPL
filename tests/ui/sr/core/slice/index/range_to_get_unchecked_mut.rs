//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

#![feature(slice_index_methods)]

use std::ops::RangeTo;
use std::slice::SliceIndex;

unsafe fn unknown_end(ptr: *mut [u8], range: RangeTo<usize>) {
    let _ = unsafe { <RangeTo<usize> as SliceIndex<[u8]>>::get_unchecked_mut(range, ptr) };
}

fn main() {
    let mut values = [1u8, 2, 3];
    let ptr = &mut values as *mut [u8];

    unsafe {
        let end_out_of_bounds = ..4usize;
        let _ = <RangeTo<usize> as SliceIndex<[u8]>>::get_unchecked_mut(end_out_of_bounds, ptr);
        //~^ unsafe_numeric_precondition

        let boundary = ..3usize;
        let _ = <RangeTo<usize> as SliceIndex<[u8]>>::get_unchecked_mut(boundary, ptr);

        unknown_end(ptr, ..2usize);
    }
}
