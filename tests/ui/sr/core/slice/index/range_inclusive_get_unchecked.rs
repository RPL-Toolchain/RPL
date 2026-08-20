//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

#![feature(slice_index_methods)]

use std::ops::RangeInclusive;
use std::slice::SliceIndex;

unsafe fn unknown_range(ptr: *const [u8], range: RangeInclusive<usize>) {
    let _ = unsafe { <RangeInclusive<usize> as SliceIndex<[u8]>>::get_unchecked(range, ptr) };
}

fn main() {
    let values = [1u8, 2, 3];
    let ptr = &values as *const [u8];

    unsafe {
        let start_after_end = 2usize..=1usize;
        let _ = <RangeInclusive<usize> as SliceIndex<[u8]>>::get_unchecked(start_after_end, ptr);
        //~^ unsafe_numeric_precondition

        let end_out_of_bounds = 1usize..=3usize;
        let _ = <RangeInclusive<usize> as SliceIndex<[u8]>>::get_unchecked(end_out_of_bounds, ptr);
        //~^ unsafe_numeric_precondition

        let valid = 1usize..=2usize;
        let _ = <RangeInclusive<usize> as SliceIndex<[u8]>>::get_unchecked(valid, ptr);

        unknown_range(ptr, 0usize..=1usize);
    }
}
