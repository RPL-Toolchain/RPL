//@revisions: inline normal
//@[normal] compile-flags: -Zinline-mir=false
pub fn base_case(p: *const u8) {
    dbg!(unsafe { *p });
    //~^ not_unsafe_ptr_arg_deref
}

pub fn cross_function(p: *const u8) {
    /// # Safety
    ///
    /// `p` must be valid to read as a `u8`
    unsafe fn deref(p: *const u8) -> u8 {
        unsafe { *p }
    }
    dbg!(unsafe { deref(p) });
    //~[inline]^ not_unsafe_ptr_arg_deref
}

pub fn cast_mutability(p: *const u8) {
    dbg!(unsafe { *p.cast_mut() });
}

#[cfg_attr(test, test)]
fn base_case_test() {
    let x = 42u8;
    base_case(&x);
}

#[cfg_attr(test, test)]
fn cross_function_test() {
    let x = 42u8;
    cross_function(&x);
}

#[cfg_attr(test, test)]
fn cast_mutability_test() {
    let x = 42u8;
    cast_mutability(&x);
}

pub(crate) fn main() {
    base_case_test();
    cross_function_test();
    cast_mutability_test();
}
