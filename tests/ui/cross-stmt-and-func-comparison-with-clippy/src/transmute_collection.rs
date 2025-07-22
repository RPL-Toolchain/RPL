//@revisions: inline normal
//@[normal] compile-flags: -Zinline-mir=false
#![expect(clippy::missing_transmute_annotations)]
use std::mem::transmute;

#[cfg_attr(test, test)]
fn base_case() {
    unsafe {
        // wrong size
        let v = transmute::<_, Vec<u32>>(vec![0u8]);
        //~^ unsound_collection_transmute
        dbg!(v);
    }
}

#[cfg_attr(test, test)]
fn cross_function() {
    /// # Safety
    ///
    /// `T` and `U` must have compatible layouts
    unsafe fn transmute_vec<T, U>(vec: Vec<T>) -> Vec<U> {
        unsafe { transmute(vec) }
    }
    unsafe {
        // wrong size
        let v = transmute_vec::<u8, u32>(vec![0u8]);
        //~[inline]^ unsound_collection_transmute

        dbg!(v);
    }
}

#[cfg_attr(test, test)]
fn cross_statement() {
    unsafe {
        let v = vec![0u8];
        //~^ unsound_collection_transmute
        // wrong size
        let v = transmute::<_, Vec<u32>>(v);
        dbg!(v);
    }
}

pub(crate) fn main() {
    base_case();
    cross_function();
    cross_statement();
}
