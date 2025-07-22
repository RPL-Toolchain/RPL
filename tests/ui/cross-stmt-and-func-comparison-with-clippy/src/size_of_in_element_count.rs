//@revisions: inline normal
//@[normal] compile-flags: -Zinline-mir=false
//@[normal] check-pass
use std::mem::size_of;
use std::ptr::copy_nonoverlapping;

#[cfg_attr(test, test)]
fn base_case() {
    const SIZE: usize = 128;
    let mut x = [2u16; SIZE];
    let mut y = [2u16; SIZE];

    // Count expression involving multiplication of size_of (Should trigger the lint)
    unsafe { copy_nonoverlapping(x.as_mut_ptr(), y.as_mut_ptr(), size_of::<u16>() * SIZE) };
    //~[inline]^ size_of_in_element_count
}

#[cfg_attr(test, test)]
fn cross_function() {
    const SIZE: usize = 128;
    let mut x = [2u16; SIZE];
    let mut y = [2u16; SIZE];

    const fn size() -> usize {
        size_of::<u16>() * SIZE
    }

    // Count expression involving multiplication of size_of (Should trigger the lint)
    unsafe { copy_nonoverlapping(x.as_mut_ptr(), y.as_mut_ptr(), size()) };
    //~[inline]^ size_of_in_element_count
}

#[cfg_attr(test, test)]
fn cross_statement() {
    const SIZE: usize = 128;
    let mut x = [2u16; SIZE];
    let mut y = [2u16; SIZE];

    let size = size_of::<u16>() * SIZE;
    //~[inline]^ size_of_in_element_count

    // Count expression involving multiplication of size_of (Should trigger the lint)
    unsafe { copy_nonoverlapping(x.as_mut_ptr(), y.as_mut_ptr(), size) };
}

pub(crate) fn main() {
    base_case();
    //~[inline]^ size_of_in_element_count
    cross_function();
    //~[inline]^ size_of_in_element_count
    cross_statement();
    //~[inline]^ size_of_in_element_count
}
