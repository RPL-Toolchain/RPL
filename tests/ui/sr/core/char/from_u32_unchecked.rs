//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

fn main() {
    unsafe {
        let too_large = 0x11_0000_u32;
        let _ = core::char::from_u32_unchecked(too_large);
        //~^ unsafe_numeric_precondition

        let surrogate = 0xD800_u32;
        let _ = core::char::from_u32_unchecked(surrogate);
        //~^ unsafe_numeric_precondition

        let valid_scalar = 0x1F600_u32;
        let _ = core::char::from_u32_unchecked(valid_scalar);
    }
}
