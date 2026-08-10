//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

use std::ptr::NonNull;

fn main() {
    unsafe {
        let nonnull_ptr = std::ptr::null_mut::<u8>();
        let _ = NonNull::new_unchecked(nonnull_ptr);
        //~^ unsafe_null_precondition

        let mut value = 1u8;
        let valid_ptr = &mut value as *mut u8;
        let _ = NonNull::new_unchecked(valid_ptr);
    }
}
