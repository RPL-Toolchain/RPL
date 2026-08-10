//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

#![feature(slice_index_methods)]

use std::ops::RangeToInclusive;
use std::slice::SliceIndex;

unsafe fn unknown_range(ptr: *mut [u8], range: RangeToInclusive<usize>) {
    let _ = unsafe { <RangeToInclusive<usize> as SliceIndex<[u8]>>::get_unchecked_mut(range, ptr) };
}

fn main() {
    let mut values = [1u8, 2, 3];
    let ptr = &mut values as *mut [u8];

    unsafe {
        let out_of_bounds = ..=3usize;
        let _ = <RangeToInclusive<usize> as SliceIndex<[u8]>>::get_unchecked_mut(out_of_bounds, ptr);
        //~^ unsafe_numeric_precondition

        let boundary = ..=2usize;
        let _ = <RangeToInclusive<usize> as SliceIndex<[u8]>>::get_unchecked_mut(boundary, ptr);

        let ordinary = ..=1usize;
        let _ = <RangeToInclusive<usize> as SliceIndex<[u8]>>::get_unchecked_mut(ordinary, ptr);

        unknown_range(ptr, ..=1usize);
    }
}
