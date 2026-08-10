//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

use std::num::NonZero;

fn main() {
    unsafe {
        let zero = 0u8;
        let _ = NonZero::<u8>::new_unchecked(zero);
        //~^ unsafe_numeric_precondition

        let zero = 0u16;
        let _ = NonZero::<u16>::new_unchecked(zero);
        //~^ unsafe_numeric_precondition

        let zero = 0u32;
        let _ = NonZero::<u32>::new_unchecked(zero);
        //~^ unsafe_numeric_precondition

        let zero = 0u64;
        let _ = NonZero::<u64>::new_unchecked(zero);
        //~^ unsafe_numeric_precondition

        let zero = 0usize;
        let _ = NonZero::<usize>::new_unchecked(zero);
        //~^ unsafe_numeric_precondition

        let zero = 0i8;
        let _ = NonZero::<i8>::new_unchecked(zero);
        //~^ unsafe_numeric_precondition

        let zero = 0i16;
        let _ = NonZero::<i16>::new_unchecked(zero);
        //~^ unsafe_numeric_precondition

        let zero = 0i32;
        let _ = NonZero::<i32>::new_unchecked(zero);
        //~^ unsafe_numeric_precondition

        let zero = 0i64;
        let _ = NonZero::<i64>::new_unchecked(zero);
        //~^ unsafe_numeric_precondition

        let zero = 0isize;
        let _ = NonZero::<isize>::new_unchecked(zero);
        //~^ unsafe_numeric_precondition

        let nonzero = 1i32;
        let _ = NonZero::<i32>::new_unchecked(nonzero);
    }
}
