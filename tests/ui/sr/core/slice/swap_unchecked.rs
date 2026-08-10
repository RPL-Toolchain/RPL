//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

#![feature(slice_swap_unchecked)]

unsafe fn unknown_indices(values: &mut [u8], a: usize, b: usize) {
    unsafe { values.swap_unchecked(a, b) };
}

fn main() {
    let mut values = [1u8, 2, 3];

    unsafe {
        let invalid_a = 3usize;
        let valid_b = 1usize;
        values.swap_unchecked(invalid_a, valid_b);
        //~^ unsafe_numeric_precondition

        let valid_a = 0usize;
        let invalid_b = 4usize;
        values.swap_unchecked(valid_a, invalid_b);
        //~^ unsafe_numeric_precondition

        let boundary = 2usize;
        let first = 0usize;
        values.swap_unchecked(boundary, first);

        let ordinary = 1usize;
        let second = 2usize;
        values.swap_unchecked(ordinary, second);

        unknown_indices(&mut values, 0usize, 1usize);
    }
}
