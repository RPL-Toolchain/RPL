//@revisions: inline normal
//@[normal] compile-flags: -Zinline-mir=false
//@compile-flags: -A rpl::uninit_assumed_init
#![expect(clippy::uninit_assumed_init)] // Non-related
use std::mem;

fn might_panic<X>(x: X) -> X {
    // in practice this would be a possibly-panicky operation
    if false {
        panic!();
    }
    x
}

#[cfg_attr(test, test)]
pub(crate) fn base_case() {
    let mut v = vec![0i32; 4];
    // the following is UB if `might_panic` panics
    unsafe {
        #[expect(invalid_value)]
        let taken_v = mem::replace(&mut v, mem::MaybeUninit::uninit().assume_init());
        //~^ mem_replace_with_uninit

        let new_v = might_panic(taken_v);
        std::mem::forget(mem::replace(&mut v, new_v));
    }
}

#[cfg_attr(test, test)]
pub(crate) fn cross_function() {
    unsafe fn uninit<T>() -> T {
        let x = mem::MaybeUninit::<T>::uninit();
        unsafe { x.assume_init() }
    }
    let mut v = vec![0i32; 4];
    // the following is UB if `might_panic` panics
    unsafe {
        let taken_v = mem::replace(&mut v, uninit());
        //~[inline]^ mem_replace_with_uninit

        let new_v = might_panic(taken_v);
        std::mem::forget(mem::replace(&mut v, new_v));
    }
}

#[cfg_attr(test, test)]
pub(crate) fn cross_statement() {
    let mut v = vec![0i32; 4];
    // the following is UB if `might_panic` panics
    unsafe {
        let u = mem::MaybeUninit::uninit();
        let u = u.assume_init();
        let taken_v = mem::replace(&mut v, u);
        //~[inline]^ mem_replace_with_uninit

        let new_v = might_panic(taken_v);
        std::mem::forget(mem::replace(&mut v, new_v));
    }
}

pub(crate) fn main() {
    base_case();
    cross_function();
    cross_statement();
}
