//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

fn main() {
    unsafe {
        let lhs = 0u8;
        let rhs = 1u8;
        let _ = lhs.unchecked_sub(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 0u16;
        let rhs = 1u16;
        let _ = lhs.unchecked_sub(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 0u32;
        let rhs = 1u32;
        let _ = lhs.unchecked_sub(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 0u64;
        let rhs = 1u64;
        let _ = lhs.unchecked_sub(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 0usize;
        let rhs = 1usize;
        let _ = lhs.unchecked_sub(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = -120i8;
        let rhs = 20i8;
        let _ = lhs.unchecked_sub(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = -32760i16;
        let rhs = 10i16;
        let _ = lhs.unchecked_sub(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = -2147483640i32;
        let rhs = 10i32;
        let _ = lhs.unchecked_sub(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = -9223372036854775800i64;
        let rhs = 10i64;
        let _ = lhs.unchecked_sub(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = -9223372036854775800isize;
        let rhs = 10isize;
        let _ = lhs.unchecked_sub(rhs);
        //~^ unsafe_numeric_precondition

        let _ = 20i8.unchecked_sub(10i8);
    }
}
