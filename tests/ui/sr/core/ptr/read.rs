//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

fn main() {
    unsafe {
        let ptr = std::ptr::null::<u8>();
        let _ = core::ptr::read(ptr);
        //~^ unsafe_null_precondition

        let mut_ptr = std::ptr::null_mut::<u8>();
        let _ = core::ptr::read(mut_ptr);
        //~^ unsafe_null_precondition

        let value = 1u8;
        let valid_ptr = &value as *const u8;
        let _ = core::ptr::read(valid_ptr);
    }
}
