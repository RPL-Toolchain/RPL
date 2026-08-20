//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

fn main() {
    unsafe {
        let mut mutable_values = [10u8, 20, 30, 40];
        let bad_mut_index = 7usize;
        let _ = mutable_values.get_unchecked_mut(bad_mut_index);
        //~^ unsafe_numeric_precondition

        let valid_mut_index = 2usize;
        let _ = mutable_values.get_unchecked_mut(valid_mut_index);
    }
}
