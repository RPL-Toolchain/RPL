//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

#![feature(array_into_iter_constructors)]

use std::array::IntoIter;
use std::mem::MaybeUninit;
use std::ops::Range;

fn buffer() -> [MaybeUninit<u8>; 3] {
    [MaybeUninit::uninit(); 3]
}

unsafe fn unknown_range(initialized: Range<usize>) {
    let values = buffer();
    let _ = unsafe { IntoIter::new_unchecked(values, initialized) };
}

fn main() {
    unsafe {
        let values = buffer();
        let start_after_end = 2usize..1usize;
        let _ = IntoIter::new_unchecked(values, start_after_end);
        //~^ unsafe_numeric_precondition

        let values = buffer();
        let end_out_of_bounds = 0usize..4usize;
        let _ = IntoIter::new_unchecked(values, end_out_of_bounds);
        //~^ unsafe_numeric_precondition

        let values = buffer();
        let valid = 1usize..3usize;
        let _ = IntoIter::new_unchecked(values, valid);

        unknown_range(0usize..2usize);
    }
}
