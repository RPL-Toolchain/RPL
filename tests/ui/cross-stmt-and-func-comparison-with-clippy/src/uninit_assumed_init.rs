//@revisions: inline normal
//@[normal] compile-flags: -Zinline-mir=false
use std::mem::MaybeUninit;

#[cfg_attr(test, test)]
fn base_case() {
    #[expect(invalid_value)]
    let x: usize = unsafe { MaybeUninit::uninit().assume_init() };
    //~^ uninit_assumed_init
    dbg!(x);
}

#[cfg_attr(test, test)]
fn cross_function_uninit() {
    fn uninit<T>() -> MaybeUninit<T> {
        MaybeUninit::uninit()
    }
    let x: usize = unsafe { uninit().assume_init() };
    //~[inline]^ uninit_assumed_init
    dbg!(x);
}

#[cfg_attr(test, test)]
fn cross_function_assume_init() {
    unsafe fn assume_init<T>(maybe_uninit: MaybeUninit<T>) -> T {
        unsafe { maybe_uninit.assume_init() }
    }
    let x: usize = unsafe { assume_init(MaybeUninit::uninit()) };
    //~[inline]^ uninit_assumed_init
    dbg!(x);
}

#[cfg_attr(test, test)]
fn cross_statement() {
    let maybe_uninit = MaybeUninit::uninit();
    let x: usize = unsafe { maybe_uninit.assume_init() };
    //~^ uninit_assumed_init
    dbg!(x);
}

#[cfg_attr(test, test)]
pub(crate) fn main() {
    base_case();
    cross_function_uninit();
    cross_function_assume_init();
    cross_statement();
}
