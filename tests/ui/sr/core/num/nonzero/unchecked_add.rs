//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false
#![feature(nonzero_ops)]

use std::num::NonZero;

fn main() {
    unsafe {
        let lhs = NonZero::<u8>::new_unchecked(250);
        let rhs = 10u8;
        let _ = lhs.unchecked_add(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = NonZero::<u16>::new_unchecked(65530);
        let rhs = 10u16;
        let _ = lhs.unchecked_add(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = NonZero::<u32>::new_unchecked(4294967290);
        let rhs = 10u32;
        let _ = lhs.unchecked_add(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = NonZero::<u64>::new_unchecked(18446744073709551610);
        let rhs = 10u64;
        let _ = lhs.unchecked_add(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = NonZero::<usize>::new_unchecked(18446744073709551610);
        let rhs = 10usize;
        let _ = lhs.unchecked_add(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = NonZero::<u8>::new_unchecked(10);
        let rhs = 20u8;
        let _ = lhs.unchecked_add(rhs);
    }
}
