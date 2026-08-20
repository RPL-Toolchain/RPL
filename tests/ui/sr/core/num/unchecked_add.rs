//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

fn main() {
    unsafe {
        let lhs = 250u8;
        let rhs = 10u8;
        let _ = lhs.unchecked_add(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 65530u16;
        let rhs = 10u16;
        let _ = lhs.unchecked_add(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 4294967290u32;
        let rhs = 10u32;
        let _ = lhs.unchecked_add(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 18446744073709551610u64;
        let rhs = 10u64;
        let _ = lhs.unchecked_add(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 18446744073709551610usize;
        let rhs = 10usize;
        let _ = lhs.unchecked_add(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 120i8;
        let rhs = 20i8;
        let _ = lhs.unchecked_add(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 32760i16;
        let rhs = 10i16;
        let _ = lhs.unchecked_add(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 2147483640i32;
        let rhs = 10i32;
        let _ = lhs.unchecked_add(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 9223372036854775800i64;
        let rhs = 10i64;
        let _ = lhs.unchecked_add(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 9223372036854775800isize;
        let rhs = 10isize;
        let _ = lhs.unchecked_add(rhs);
        //~^ unsafe_numeric_precondition

        let _ = 10u8.unchecked_add(20u8);
    }
}
