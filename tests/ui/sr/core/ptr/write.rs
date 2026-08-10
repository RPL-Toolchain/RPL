//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

fn main() {
    unsafe {
        let write_ptr = std::ptr::null_mut::<u8>();
        core::ptr::write(write_ptr, 1);
        //~^ unsafe_null_precondition

        let mut value = 0u8;
        let valid_ptr = &mut value as *mut u8;
        core::ptr::write(valid_ptr, 1);
    }
}
