fn unchecked<T>(ptr: *const T, index: usize, length: usize) -> *const T {
    unsafe {
        let mut p = ptr;
        p = p.add(index);
        //~^ERROR: it is an undefined behavior to offset a pointer using an unchecked integer
        p
    }
}

fn unchecked_slice<T>(slice: &[T], index: usize) -> *const T {
    let mut p = slice.as_ptr();
    let length = slice.len();
    unsafe {
        p = p.add(index);
        //~^ERROR: it is an undefined behavior to offset a pointer using an unchecked integer
        &*p
    }
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn slice_end<T>(slice: &[T]) -> *const T {
    let p = slice.as_ptr();
    let length = slice.len();
    unsafe { p.add(length) }
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn slice_at<T>(slice: &[T], index: usize) -> *const T {
    let p = slice.as_ptr();
    let length = slice.len();
    assert!(index < length);
    unsafe { p.add(length) }
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn vec_iter(vec: &Vec<usize>) -> usize {
    let mut x = 0;
    for i in vec {
        x += 1000000007 % (*i + 1);
    }
    x
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn vec_iter_mut(vec: &mut Vec<usize>) -> usize {
    let mut x = 0;
    for i in vec.iter_mut() {
        x += 1000000007 % (*i + 1);
        *i += 1;
    }
    x
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn slice_iter(vec: &[usize]) -> usize {
    let mut x = 0;
    for i in vec {
        x += 1000000007 % (*i + 1);
    }
    x
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn slice_iter_mut(vec: &mut [usize]) -> usize {
    let mut x = 0;
    for i in vec.iter_mut() {
        x += 1000000007 % (*i + 1);
        *i += 1;
    }
    x
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn checked_lt<T>(slice: &[T], index: usize) -> &T {
    let mut p: *const T = slice.as_ptr();
    let length: usize = slice.len();
    assert!(index < length);
    unsafe {
        p = p.add(index);
        &*p
    }
}

fn checked_le<T>(ptr: *const T, index: usize, length: usize) -> *const T {
    unsafe {
        let mut p = ptr;
        // Though `index + 1` is moved in MIR, the negative pattern is still detected, so no false positive here
        assert!(index + 1 <= length);
        p = p.add(index);
        p
    }
}

fn checked_le_1<T>(ptr: *const T, index: usize, right: usize) -> *const T {
    unsafe {
        let mut p = ptr;
        assert!(index <= right);
        p = p.add(index);
        p
    }
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn safe_vec_deref<T>(slice: &Vec<T>) -> &[T] {
    &*slice
    // This is safe because the length will be checked at runtime
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn safe_vec_deref_mut<T>(slice: &mut Vec<T>) -> &mut [T] {
    &mut *slice
    // This is safe because the length will be checked at runtime
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn safe_slice_range_from<T>(slice: &[T]) -> &[T] {
    &slice[1..]
    // This is safe because the length will be checked at runtime
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn safe_slice_mut_range_from<T>(slice: &mut [T]) -> &mut [T] {
    &mut slice[1..]
    // This is safe because the length will be checked at runtime
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn safe_slice_range_to<T>(slice: &[T]) -> &[T] {
    &slice[..2]
    // This is safe because the length will be checked at runtime
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn safe_slice_mut_range_to<T>(slice: &mut [T]) -> &mut [T] {
    &mut slice[..2]
    // This is safe because the length will be checked at runtime
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn safe_slice_range_full<T>(slice: &[T]) -> &[T] {
    &slice[..]
    // This is safe as there is no out-of-bounds access
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn safe_slice_mut_range_full<T>(slice: &mut [T]) -> &mut [T] {
    &mut slice[..]
    // This is safe as there is no out-of-bounds access
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn safe_vec_ref_range_full<T>(slice: &Vec<T>) -> &[T] {
    &slice[..]
    // This is safe as there is no out-of-bounds access
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn safe_vec_ref_mut_range_full<T>(slice: &mut Vec<T>) -> &mut [T] {
    &mut slice[..]
    // This is safe as there is no out-of-bounds access
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn safe_vec_range_full<T>() {
    let v = Vec::new();
    let slice: &[T] = &v[..];
    // This is safe as there is no out-of-bounds access
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn safe_vec_mut_range_full<T>() {
    let mut v = Vec::new();
    let slice: &mut [T] = &mut v[..];
    // This is safe as there is no out-of-bounds access
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn safe_array_in_bound<T>(slice: &[T; 2]) -> &T {
    let ptr = slice.as_ptr();
    unsafe { &*ptr.add(1) }
    // This is safe because the length of the slice is known at compile time
    // and the index is guaranteed to be less than the length.
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn unsafe_array_out_of_bound_1<T>(slice: &[T; 2]) -> &T {
    let ptr = slice.as_ptr();
    unsafe { &*ptr.add(2) }
    //~^ERROR: it is an undefined behavior to offset a pointer using an unchecked integer
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn unsafe_array_out_of_bound_2<T>(slice: &[T; 2]) -> &T {
    let ptr = slice.as_ptr();
    unsafe { &*ptr.add(4) }
    //~^ERROR: it is an undefined behavior to offset a pointer using an unchecked integer
}

fn safe_unchecked_2_const_rem<T, const N: usize>(slice: &[T; N], index: usize) -> &T {
    let ptr = slice.as_ptr();
    unsafe { &*ptr.add(index % N) }
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn safe_unchecked_2_const<T, const N: usize>(slice: &[T; N]) -> &T {
    let ptr = slice.as_ptr();
    unsafe { &*ptr.add(N) }
    //~^ERROR: it is an undefined behavior to offset a pointer using an unchecked integer
}

fn safe_unchecked_2_const_literal_2<T>(slice: &[T; 2], index: usize) -> &T {
    let ptr = slice.as_ptr();
    unsafe { &*ptr.add(index % 2) }
}

fn safe_unchecked_2_const_literal_0<T>(slice: &[T; 0], index: usize) -> &T {
    let ptr = slice.as_ptr();
    unsafe { &*ptr.add(index % 0) }
    //~^ERROR: this operation will panic at runtime
}

fn safe_unchecked_2_mismatched<T>(slice: &[T], index: usize) -> &T {
    let ptr = slice.as_ptr();
    unsafe { &*ptr.add(index % 2) }
    //FIXME: this is a false negative
}

fn safe_unchecked_2_const_literal_2_3_mismatched<T>(slice: &[T; 2], index: usize) -> &T {
    let ptr = slice.as_ptr();
    unsafe { &*ptr.add(index % 3) }
    //FIXME: this is a false negative
}

fn safe_unchecked_2<T>(slice: &[T], index: usize) -> &T {
    let ptr = slice.as_ptr();
    let length = slice.len();
    unsafe { &*ptr.add(index % length) }
}

fn safe_unchecked_without_offset<T>(slice: &[T; 2]) -> &T {
    &slice[1]
}

unsafe fn unsafe_unchecked_in_unsafe<T>(p: *const T) -> *const T {
    // Do anything you want with `p`, as it's in an `unsafe` function
    unsafe { p.add(1) }
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn unsafe_unchecked_in_safe<T>(p: *const T) -> *const T {
    // Sorry, it's in a safe function :(
    unsafe { p.add(1) }
    //~^ERROR: it is an undefined behavior to offset a pointer using an unchecked integer
}
