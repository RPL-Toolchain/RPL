//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

use std::ffi::CStr;

fn main() {
    unsafe {
        let missing = [b'a', b'b', b'c'];
        let _ = CStr::from_bytes_with_nul_unchecked(&missing);
        //~^ unsafe_cstr_bytes_precondition

        let interior = [b'a', 0, b'b', 0];
        let _ = CStr::from_bytes_with_nul_unchecked(&interior);
        //~^ unsafe_cstr_bytes_precondition

        let empty = [];
        let _ = CStr::from_bytes_with_nul_unchecked(&empty);
        //~^ unsafe_cstr_bytes_precondition

        let mut runtime = [b'a', b'b', 0];
        runtime[1] = 0;
        let _ = CStr::from_bytes_with_nul_unchecked(&runtime);
        //~^ unsafe_cstr_bytes_precondition

        let valid = [b'v', b'a', b'l', b'i', b'd', 0];
        let _ = CStr::from_bytes_with_nul_unchecked(&valid);
    }
}
