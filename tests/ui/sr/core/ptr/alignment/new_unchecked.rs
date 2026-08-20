//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

#![feature(ptr_alignment_type)]

use std::ptr::Alignment;

fn main() {
    unsafe {
        let zero_align = 0usize;
        let _ = Alignment::new_unchecked(zero_align);
        //~^ unsafe_numeric_precondition

        let non_power_of_two_align = 3usize;
        let _ = Alignment::new_unchecked(non_power_of_two_align);
        //~^ unsafe_numeric_precondition

        let valid_align = 8usize;
        let _ = Alignment::new_unchecked(valid_align);
    }
}
