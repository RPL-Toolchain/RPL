//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

#![feature(slice_index_methods)]

use std::ops::Range;
use std::slice::SliceIndex;

fn main() {
    let values = [1u8, 2, 3];
    let slice: &[u8] = &values;
    let ptr = slice as *const [u8];

    unsafe {
        let start_after_end = 2usize..1usize;
        let _ = <Range<usize> as SliceIndex<[u8]>>::get_unchecked(start_after_end, ptr);
        //~^ unsafe_numeric_precondition

        let end_out_of_bounds = 1usize..4usize;
        let _ = <Range<usize> as SliceIndex<[u8]>>::get_unchecked(end_out_of_bounds, ptr);
        //~^ unsafe_numeric_precondition

        let valid_range = 1usize..3usize;
        let _ = <Range<usize> as SliceIndex<[u8]>>::get_unchecked(valid_range, ptr);

        let raw_data = values.as_ptr();
        let raw_len = 3usize;
        let raw_ptr = std::ptr::slice_from_raw_parts(raw_data, raw_len);
        let raw_end_out_of_bounds = 0usize..4usize;
        let _ = <Range<usize> as SliceIndex<[u8]>>::get_unchecked(raw_end_out_of_bounds, raw_ptr);
        //~^ unsafe_numeric_precondition
    }

    dangling_raw_slice_pointer();
}

fn dangling_raw_slice_pointer() {
    let data: *const u8;
    {
        let boxed = Box::new(1u8);
        data = &*boxed as *const u8;
    }
    let ptr = std::ptr::slice_from_raw_parts(data, 1usize);

    unsafe {
        let valid_range = 0usize..1usize;
        let _ = <Range<usize> as SliceIndex<[u8]>>::get_unchecked(valid_range, ptr);
        //~^ unsafe_dangling_pointer_precondition
    }
}
