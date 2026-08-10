//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

#![feature(slice_index_methods)]

use std::slice::SliceIndex;

fn main() {
    let mut values = [1u8, 2, 3];
    let slice: &mut [u8] = &mut values;
    let ptr = slice as *mut [u8];

    unsafe {
        let bad_index = 3usize;
        let _ = <usize as SliceIndex<[u8]>>::get_unchecked_mut(bad_index, ptr);
        //~^ unsafe_numeric_precondition

        let valid_index = 2usize;
        let _ = <usize as SliceIndex<[u8]>>::get_unchecked_mut(valid_index, ptr);
    }

    let mut raw_values = [1u8, 2, 3];
    unsafe {
        let raw_data = raw_values.as_mut_ptr();
        let raw_len = 3usize;
        let raw_ptr = std::ptr::slice_from_raw_parts_mut(raw_data, raw_len);
        let bad_index_from_raw_parts = 4usize;
        let _ = <usize as SliceIndex<[u8]>>::get_unchecked_mut(bad_index_from_raw_parts, raw_ptr);
        //~^ unsafe_numeric_precondition
    }

    dangling_raw_slice_pointer();
}

fn dangling_raw_slice_pointer() {
    let data: *mut u8;
    {
        let mut boxed = Box::new(1u8);
        data = &mut *boxed as *mut u8;
    }
    let ptr = std::ptr::slice_from_raw_parts_mut(data, 1usize);

    unsafe {
        let valid_index = 0usize;
        let _ = <usize as SliceIndex<[u8]>>::get_unchecked_mut(valid_index, ptr);
        //~^ unsafe_dangling_pointer_precondition
    }
}
