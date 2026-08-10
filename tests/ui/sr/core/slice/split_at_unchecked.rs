//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

fn main() {
    unsafe {
        let split_values = [1u8, 2, 3];
        let bad_mid = 5usize;
        let _ = split_values.split_at_unchecked(bad_mid);
        //~^ unsafe_numeric_precondition

        let valid_mid = 2usize;
        let _ = split_values.split_at_unchecked(valid_mid);
    }
}
