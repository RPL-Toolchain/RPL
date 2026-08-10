//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false
//@rustc-env: RPL_PATS=docs/patterns-pest/sr/core/ffi/c_str/from_ptr.rpl
use std::ffi::{c_char, CStr};

unsafe fn unknown_pointer(ptr: *const c_char) {
    let _ = unsafe { CStr::from_ptr(ptr) };
}

fn main() {
    unsafe {
        let cstr_ptr = std::ptr::null::<c_char>();
        let _ = CStr::from_ptr(cstr_ptr);
        //~^ unsafe_null_precondition

        let valid_bytes = b"valid\0";
        let _ = CStr::from_ptr(valid_bytes.as_ptr().cast::<c_char>());

        let missing_nul = [b'n', b'o', b'p', b'e'];
        let missing_nul_ptr = missing_nul.as_ptr().cast::<c_char>();
        let _ = CStr::from_ptr(missing_nul_ptr);
        //~^ unsafe_cstr_terminator_precondition

        let interior_nul = [b'o', b'k', 0, b'x'];
        let interior_nul_ptr = interior_nul.as_ptr().cast::<c_char>();
        let _ = CStr::from_ptr(interior_nul_ptr);

        let empty: [u8; 0] = [];
        let empty_ptr = empty.as_ptr().cast::<c_char>();
        let _ = CStr::from_ptr(empty_ptr);
        //~^ unsafe_cstr_terminator_precondition

        unknown_pointer(valid_bytes.as_ptr().cast::<c_char>());
    }
}
