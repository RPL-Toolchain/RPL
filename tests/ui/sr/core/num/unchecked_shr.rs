//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

#![feature(unchecked_shifts)]

fn main() {
    unsafe {
        let lhs = 128u8;
        let rhs = 8u32;
        let _ = lhs.unchecked_shr(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 128u16;
        let rhs = 16u32;
        let _ = lhs.unchecked_shr(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 128u32;
        let rhs = 32u32;
        let _ = lhs.unchecked_shr(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 128u64;
        let rhs = 64u32;
        let _ = lhs.unchecked_shr(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 128usize;
        let rhs = 64u32;
        let _ = lhs.unchecked_shr(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = -128i8;
        let rhs = 8u32;
        let _ = lhs.unchecked_shr(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = -128i16;
        let rhs = 16u32;
        let _ = lhs.unchecked_shr(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = -128i32;
        let rhs = 32u32;
        let _ = lhs.unchecked_shr(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = -128i64;
        let rhs = 64u32;
        let _ = lhs.unchecked_shr(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = -128isize;
        let rhs = 64u32;
        let _ = lhs.unchecked_shr(rhs);
        //~^ unsafe_numeric_precondition

        let _ = 128u8.unchecked_shr(7u32);
    }
}
