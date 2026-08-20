//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

use core::alloc::Layout;

fn main() {
    unsafe {
        let size = 8usize;
        let zero_align = 0usize;
        let _ = Layout::from_size_align_unchecked(size, zero_align);
        //~^ unsafe_numeric_precondition

        let non_power_of_two_align = 3usize;
        let _ = Layout::from_size_align_unchecked(size, non_power_of_two_align);
        //~^ unsafe_numeric_precondition

        let oversized = 9_223_372_036_854_775_807usize;
        let align = 2usize;
        let _ = Layout::from_size_align_unchecked(oversized, align);
        //~^ unsafe_numeric_precondition

        let valid_size = 16usize;
        let valid_align = 8usize;
        let _ = Layout::from_size_align_unchecked(valid_size, valid_align);
    }
}
