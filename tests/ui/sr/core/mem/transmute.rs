//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false
#![allow(invalid_value)]
#![allow(rpl::transmute_int_to_non_zero, rpl::transmuting_type_to_bool)]

use std::mem;
use std::num::NonZeroU8;

unsafe fn unknown_values(bits8: u8, bits32: u32) {
    let _: bool = unsafe { mem::transmute(bits8) };
    let _: char = unsafe { mem::transmute(bits32) };
}

fn main() {
    unsafe {
        let invalid_bool = 2u8;
        let _: bool = mem::transmute(invalid_bool);
        //~^ unsafe_transmute_value_precondition

        let valid_bool = 1u8;
        let _: bool = mem::transmute(valid_bool);

        let surrogate = 0xD800u32;
        let _: char = mem::transmute(surrogate);
        //~^ unsafe_transmute_value_precondition

        let above_char_max = 0x11_0000u32;
        let _: char = mem::transmute(above_char_max);
        //~^ unsafe_transmute_value_precondition

        let valid_char = 0x10_FFFFu32;
        let _: char = mem::transmute(valid_char);

        let zero = 0u8;
        let _: NonZeroU8 = mem::transmute(zero);
        //~^ unsafe_transmute_value_precondition

        let nonzero = 7u8;
        let _: NonZeroU8 = mem::transmute(nonzero);

        unknown_values(valid_bool, valid_char);
    }
}
