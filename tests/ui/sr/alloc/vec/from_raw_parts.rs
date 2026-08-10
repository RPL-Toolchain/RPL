//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

fn main() {
    unsafe {
        let vec_ptr = std::ptr::null_mut::<u8>();
        let _ = Vec::from_raw_parts(vec_ptr, 0, 0);
        //~^ unsafe_null_precondition

        let valid_ptr = std::ptr::NonNull::<u8>::dangling().as_ptr();
        let _ = Vec::from_raw_parts(valid_ptr, 0, 0);
    }
}
