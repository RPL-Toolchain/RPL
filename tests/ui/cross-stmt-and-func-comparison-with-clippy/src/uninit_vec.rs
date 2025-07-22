//@revisions: inline normal
//@[normal] compile-flags: -Zinline-mir=false
//@[inline] check-pass
#[cfg_attr(test, test)]
fn base_case() {
    // with_capacity() -> set_len() should be detected
    let mut vec: Vec<u8> = Vec::with_capacity(1000);
    //~[normal]^ uninit_vec

    unsafe {
        vec.set_len(200);
        //~[normal]^ set_len_uninitialized
    }

    dbg!(vec[0]);
}

#[cfg_attr(test, test)]
fn cross_function_with_capacity() {
    fn with_capacity<T>(capacity: usize) -> Vec<T> {
        Vec::with_capacity(capacity)
    }
    // with_capacity() -> set_len() should be detected
    let mut vec: Vec<u8> = with_capacity(1000);
    //FIXME: ~^ uninit_vec

    unsafe {
        vec.set_len(200);
    }

    dbg!(vec[0]);
}

#[cfg_attr(test, test)]
fn cross_function_set_len() {
    unsafe fn set_len<T>(v: &mut Vec<T>, len: usize) {
        unsafe {
            v.set_len(len);
        }
    }
    // with_capacity() -> set_len() should be detected
    let mut vec: Vec<u8> = Vec::with_capacity(1000);
    //FIXME: ~^ uninit_vec

    unsafe {
        set_len(&mut vec, 200);
    }

    dbg!(vec[0]);
}

fn cross_function_both() {
    fn with_capacity<T>(capacity: usize) -> Vec<T> {
        Vec::with_capacity(capacity)
    }
    unsafe fn set_len<T>(v: &mut Vec<T>, len: usize) {
        unsafe {
            v.set_len(len);
        }
    }
    // with_capacity() -> set_len() should be detected
    let mut vec: Vec<u8> = with_capacity(1000);
    //FIXME: ~^ uninit_vec

    unsafe {
        set_len(&mut vec, 200);
    }

    dbg!(vec[0]);
}

pub(crate) fn main() {
    base_case();
    cross_function_with_capacity();
    cross_function_set_len();
    cross_function_both();
}
