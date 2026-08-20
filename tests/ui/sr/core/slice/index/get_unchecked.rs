//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

#![feature(slice_index_methods)]

use std::slice::SliceIndex;

fn main() {
    let values = [1u8, 2, 3];
    let slice: &[u8] = &values;
    let ptr = slice as *const [u8];

    unsafe {
        let bad_index = 3usize;
        let _ = <usize as SliceIndex<[u8]>>::get_unchecked(bad_index, ptr);
        //~^ unsafe_numeric_precondition

        let valid_index = 2usize;
        let _ = <usize as SliceIndex<[u8]>>::get_unchecked(valid_index, ptr);

        let raw_data = values.as_ptr();
        let raw_len = 3usize;
        let raw_ptr = std::ptr::slice_from_raw_parts(raw_data, raw_len);
        let bad_index_from_raw_parts = 4usize;
        let _ = <usize as SliceIndex<[u8]>>::get_unchecked(bad_index_from_raw_parts, raw_ptr);
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
        let valid_index = 0usize;
        let _ = <usize as SliceIndex<[u8]>>::get_unchecked(valid_index, ptr);
        //~^ unsafe_dangling_pointer_precondition
    }
}
