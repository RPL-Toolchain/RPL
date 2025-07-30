//@revisions: inline normal
//@[normal] compile-flags: -Zinline-mir=false

/// Base cases
#[cfg_attr(test, test)]
fn base_case() {
    unsafe {
        let mut m = ();
        let m = &raw mut m;

        let n = m.offset(0);
        //~^ zst_offset
        dbg!(n);

        let n = m.wrapping_add(0);
        //~^ zst_offset
        dbg!(n);
    }
}

/// Cross-function cases
#[cfg_attr(test, test)]
fn cross_function() {
    unsafe fn offset<T>(m: *mut T, n: isize) -> *mut T {
        unsafe { m.offset(n) }
    }
    fn wrapping_add<T>(m: *mut T, n: usize) -> *mut T {
        m.wrapping_add(n)
        //~[inline]^ unchecked_pointer_offset
    }
    unsafe {
        let mut m = ();
        let m = &raw mut m;

        let n = offset(m, 0);
        //~[inline]^ zst_offset
        dbg!(n);

        let n = wrapping_add(m, 0);
        //~[inline]^ zst_offset
        dbg!(n);
    }
}

pub(crate) fn main() {
    base_case();
    cross_function();
}
