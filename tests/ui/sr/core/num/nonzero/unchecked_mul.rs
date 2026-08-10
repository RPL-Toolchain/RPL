//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false
#![feature(nonzero_ops)]

use std::num::NonZero;

fn main() {
    unsafe {
        let lhs = NonZero::<u8>::new_unchecked(20);
        let rhs = NonZero::<u8>::new_unchecked(20);
        let _ = lhs.unchecked_mul(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = NonZero::<u16>::new_unchecked(500);
        let rhs = NonZero::<u16>::new_unchecked(500);
        let _ = lhs.unchecked_mul(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = NonZero::<u32>::new_unchecked(70000);
        let rhs = NonZero::<u32>::new_unchecked(70000);
        let _ = lhs.unchecked_mul(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = NonZero::<u64>::new_unchecked(5000000000);
        let rhs = NonZero::<u64>::new_unchecked(5000000000);
        let _ = lhs.unchecked_mul(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = NonZero::<usize>::new_unchecked(5000000000);
        let rhs = NonZero::<usize>::new_unchecked(5000000000);
        let _ = lhs.unchecked_mul(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = NonZero::<i8>::new_unchecked(100);
        let rhs = NonZero::<i8>::new_unchecked(2);
        let _ = lhs.unchecked_mul(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = NonZero::<i16>::new_unchecked(20000);
        let rhs = NonZero::<i16>::new_unchecked(2);
        let _ = lhs.unchecked_mul(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = NonZero::<i32>::new_unchecked(2000000000);
        let rhs = NonZero::<i32>::new_unchecked(2);
        let _ = lhs.unchecked_mul(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = NonZero::<i64>::new_unchecked(5000000000);
        let rhs = NonZero::<i64>::new_unchecked(5000000000);
        let _ = lhs.unchecked_mul(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = NonZero::<isize>::new_unchecked(5000000000);
        let rhs = NonZero::<isize>::new_unchecked(5000000000);
        let _ = lhs.unchecked_mul(rhs);
        //~^ unsafe_numeric_precondition

        let lhs = NonZero::<i16>::new_unchecked(100);
        let rhs = NonZero::<i16>::new_unchecked(2);
        let _ = lhs.unchecked_mul(rhs);
    }
}
