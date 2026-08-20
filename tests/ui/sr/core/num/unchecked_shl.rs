//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

#![feature(unchecked_shifts)]

fn main() {
    unsafe {
        let lhs = 1u8;
        let rhs = 8u32;
        let _ = lhs.unchecked_shl(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 1u16;
        let rhs = 16u32;
        let _ = lhs.unchecked_shl(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 1u32;
        let rhs = 32u32;
        let _ = lhs.unchecked_shl(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 1u64;
        let rhs = 64u32;
        let _ = lhs.unchecked_shl(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 1usize;
        let rhs = 64u32;
        let _ = lhs.unchecked_shl(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 1i8;
        let rhs = 8u32;
        let _ = lhs.unchecked_shl(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 1i16;
        let rhs = 16u32;
        let _ = lhs.unchecked_shl(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 1i32;
        let rhs = 32u32;
        let _ = lhs.unchecked_shl(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 1i64;
        let rhs = 64u32;
        let _ = lhs.unchecked_shl(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 1isize;
        let rhs = 64u32;
        let _ = lhs.unchecked_shl(rhs);
        //~^ unsafe_numeric_precondition

        let _ = 1u8.unchecked_shl(7u32);
    }
}
