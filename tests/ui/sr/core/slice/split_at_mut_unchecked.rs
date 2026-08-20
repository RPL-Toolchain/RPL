//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

fn main() {
    unsafe {
        let mut split_mut_values = [1u8, 2, 3];
        let bad_mut_mid = 6usize;
        let _ = split_mut_values.split_at_mut_unchecked(bad_mut_mid);
        //~^ unsafe_numeric_precondition

        let valid_mut_mid = 2usize;
        let _ = split_mut_values.split_at_mut_unchecked(valid_mut_mid);
    }
}
