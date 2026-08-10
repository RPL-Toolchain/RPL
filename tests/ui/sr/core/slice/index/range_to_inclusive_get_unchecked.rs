//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

#![feature(slice_index_methods)]

use std::ops::RangeToInclusive;
use std::slice::SliceIndex;

unsafe fn unknown_range(ptr: *const [u8], range: RangeToInclusive<usize>) {
    let _ = unsafe { <RangeToInclusive<usize> as SliceIndex<[u8]>>::get_unchecked(range, ptr) };
}

fn main() {
    let values = [1u8, 2, 3];
    let ptr = &values as *const [u8];

    unsafe {
        let out_of_bounds = ..=3usize;
        let _ = <RangeToInclusive<usize> as SliceIndex<[u8]>>::get_unchecked(out_of_bounds, ptr);
        //~^ unsafe_numeric_precondition

        let boundary = ..=2usize;
        let _ = <RangeToInclusive<usize> as SliceIndex<[u8]>>::get_unchecked(boundary, ptr);

        let ordinary = ..=1usize;
        let _ = <RangeToInclusive<usize> as SliceIndex<[u8]>>::get_unchecked(ordinary, ptr);

        unknown_range(ptr, ..=1usize);
    }
}
