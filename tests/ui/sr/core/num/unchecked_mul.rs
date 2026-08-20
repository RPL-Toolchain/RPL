//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

fn main() {
    unsafe {
        let lhs = 20u8;
        let rhs = 20u8;
        let _ = lhs.unchecked_mul(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 500u16;
        let rhs = 500u16;
        let _ = lhs.unchecked_mul(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 70000u32;
        let rhs = 70000u32;
        let _ = lhs.unchecked_mul(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 5000000000u64;
        let rhs = 5000000000u64;
        let _ = lhs.unchecked_mul(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 5000000000usize;
        let rhs = 5000000000usize;
        let _ = lhs.unchecked_mul(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 100i8;
        let rhs = 2i8;
        let _ = lhs.unchecked_mul(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 20000i16;
        let rhs = 2i16;
        let _ = lhs.unchecked_mul(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 2000000000i32;
        let rhs = 2i32;
        let _ = lhs.unchecked_mul(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 5000000000i64;
        let rhs = 5000000000i64;
        let _ = lhs.unchecked_mul(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = 5000000000isize;
        let rhs = 5000000000isize;
        let _ = lhs.unchecked_mul(rhs);
        //~^ unsafe_numeric_precondition

        let _ = 100i16.unchecked_mul(2i16);
    }
}
