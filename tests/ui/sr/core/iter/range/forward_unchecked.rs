//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

#![feature(step_trait)]

use core::iter::Step;

fn main() {
    unsafe {
        let unsigned_start = 250_u8;
        let unsigned_count = 10_usize;
        let _ = <u8 as Step>::forward_unchecked(unsigned_start, unsigned_count);
        //~^ unsafe_numeric_precondition

        let u16_start = 65_530_u16;
        let u16_count = 10_usize;
        let _ = <u16 as Step>::forward_unchecked(u16_start, u16_count);
        //~^ unsafe_numeric_precondition

        let u32_start = 4_294_967_290_u32;
        let u32_count = 10_usize;
        let _ = <u32 as Step>::forward_unchecked(u32_start, u32_count);
        //~^ unsafe_numeric_precondition

        let u64_start = 18_446_744_073_709_551_610_u64;
        let u64_count = 10_usize;
        let _ = <u64 as Step>::forward_unchecked(u64_start, u64_count);
        //~^ unsafe_numeric_precondition

        let signed_start = 120_i8;
        let signed_count = 10_usize;
        let _ = <i8 as Step>::forward_unchecked(signed_start, signed_count);
        //~^ unsafe_numeric_precondition

        let i16_start = 32_760_i16;
        let i16_count = 10_usize;
        let _ = <i16 as Step>::forward_unchecked(i16_start, i16_count);
        //~^ unsafe_numeric_precondition

        let i32_start = 2_147_483_640_i32;
        let i32_count = 10_usize;
        let _ = <i32 as Step>::forward_unchecked(i32_start, i32_count);
        //~^ unsafe_numeric_precondition

        let i64_start = 9_223_372_036_854_775_800_i64;
        let i64_count = 10_usize;
        let _ = <i64 as Step>::forward_unchecked(i64_start, i64_count);
        //~^ unsafe_numeric_precondition

        let usize_start = 18_446_744_073_709_551_615_usize;
        let one = 1_usize;
        let _ = <usize as Step>::forward_unchecked(usize_start, one);
        //~^ unsafe_numeric_precondition

        let isize_start = 9_223_372_036_854_775_807_isize;
        let _ = <isize as Step>::forward_unchecked(isize_start, one);
        //~^ unsafe_numeric_precondition

        let char_max_start = '\u{10FFFF}';
        let _ = <char as Step>::forward_unchecked(char_max_start, one);
        //~^ unsafe_numeric_precondition

        let char_surrogate_skip_start = '\0';
        let char_surrogate_skip_count = 0x10F800_usize;
        let _ = <char as Step>::forward_unchecked(char_surrogate_skip_start, char_surrogate_skip_count);
        //~^ unsafe_numeric_precondition

        let valid_unsigned_start = 100_u16;
        let valid_unsigned_count = 20_usize;
        let _ = <u16 as Step>::forward_unchecked(valid_unsigned_start, valid_unsigned_count);

        let valid_signed_start = -120_i8;
        let valid_signed_count = 200_usize;
        let _ = <i8 as Step>::forward_unchecked(valid_signed_start, valid_signed_count);

        let valid_char_start = 'a';
        let valid_char_count = 1_usize;
        let _ = <char as Step>::forward_unchecked(valid_char_start, valid_char_count);
    }
}
