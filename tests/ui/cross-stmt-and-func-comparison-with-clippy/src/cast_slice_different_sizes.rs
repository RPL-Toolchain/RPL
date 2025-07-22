//@revisions: inline normal
//@[normal] compile-flags: -Zinline-mir=false
#[cfg_attr(test, test)]
pub(crate) fn base_case() {
    let x: [i32; 3] = [1_i32, 2, 3];
    let r_x = &x;
    // Check casting through multiple bindings
    // Because it's separate, it does not check the cast back to something of the same size
    let a = r_x as *const [i32];
    let b = a as *const [u64];
    //~^ cast_slice_different_sizes
    dbg!(b);
}
#[cfg_attr(test, test)]
pub(crate) fn cross_function() {
    fn cast_slice<T, U>(slice: *const [T]) -> *const [U] {
        slice as *const [U]
        // unsafe {
        //     std::slice::from_raw_parts(
        //         slice.as_ptr() as *const U,
        //         slice.len() * std::mem::size_of::<T>() / std::mem::size_of::<U>(),
        //     )
        // }
    }
    let x: [i32; 3] = [1_i32, 2, 3];
    let r_x = &x;
    // Check casting through multiple bindings
    // Because it's separate, it does not check the cast back to something of the same size
    let b: *const [u64] = cast_slice(r_x);
    //~[inline]^ cast_slice_different_sizes
    dbg!(b);
}

pub(crate) fn main() {
    base_case();
    cross_function();
}
