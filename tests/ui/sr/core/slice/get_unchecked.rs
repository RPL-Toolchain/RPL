//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

fn main() {
    unsafe {
        let values = [10u8, 20, 30, 40];
        let bad_index = 8usize;
        let _ = values.get_unchecked(bad_index);
        //~^ unsafe_numeric_precondition

        let valid_index = 2usize;
        let _ = values.get_unchecked(valid_index);
    }
}
