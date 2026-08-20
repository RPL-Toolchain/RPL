//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false
#![allow(invalid_value)]
#![allow(rpl::transmuting_type_to_bool)]

use std::mem;

unsafe fn unknown_value(bits: &u8) {
    let _: bool = unsafe { mem::transmute_copy(bits) };
}

fn main() {
    unsafe {
        let invalid_bool = 3u8;
        let _: bool = mem::transmute_copy(&invalid_bool);
        //~^ unsafe_transmute_copy_value_precondition

        let valid_bool = 0u8;
        let _: bool = mem::transmute_copy(&valid_bool);

        unknown_value(&valid_bool);
    }
}
