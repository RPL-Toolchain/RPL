//@ revisions: inline normal
//@[inline] compile-flags: -Z inline-mir=true
//@[normal] compile-flags: -Z inline-mir=false

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
pub fn swap_index_for_enumerate_(bla: impl ExactSizeIterator<Item = u32>) -> Vec<u32> {
    let len = bla.len();
    let mut vec = Vec::with_capacity(len); //~[normal] uninit_vec
    let arr: &mut [u32] = unsafe { std::slice::from_raw_parts_mut(vec.as_mut_ptr(), bla.len()) };
    //~[normal]^ ERROR: it violates the precondition of `std::slice::from_raw_parts_mut` to create a slice from a `Vec` that is not initialized yet
    for (i, a) in bla.enumerate() {
        arr[a as usize] = i as u32;
    }

    unsafe {
        vec.set_len(len);
        //~^ ERROR: it is unsound to trust return value of `std::iter::ExactSizeIterator::len` and pass it to an unsafe function like `std::vec::Vec::set_len`, which may leak uninitialized memory
        //~[normal]| ERROR: it violates the precondition of `Vec::set_len` to extend a `Vec`'s length without initializing its content in advance
    }
    vec
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
pub fn with_capacity_write_from_raw_parts(bla: impl ExactSizeIterator<Item = u32>) -> Vec<u32> {
    let len = bla.len();
    let mut vec: Vec<u32> = Vec::with_capacity(len);
    let ptr = vec.as_mut_ptr();
    for (i, a) in bla.enumerate() {
        unsafe {
            ptr.add(a as usize).write(i as u32);
        }
    }
    let arr: &mut [u32] = unsafe { std::slice::from_raw_parts_mut(vec.as_mut_ptr(), len) };
    //~[normal]^ERROR: it violates the precondition of `std::slice::from_raw_parts_mut` to create a slice from a `Vec` that is not initialized yet
    // not all elements of `vec` are guaranteed to be initialized
    vec
}

pub fn swap_index_range_loop_next(bla: std::ops::Range<u32>) -> Vec<u32> {
    let len = bla.len();
    let mut vec = Vec::with_capacity(len); //~[normal] uninit_vec
    let arr: &mut [u32] = unsafe { std::slice::from_raw_parts_mut(vec.as_mut_ptr(), bla.len()) };
    //~[normal]^ ERROR: it violates the precondition of `std::slice::from_raw_parts_mut` to create a slice from a `Vec` that is not initialized yet
    /* for (i, a) in bla.enumerate() {
        arr[a as usize] = i as u32;
    } */
    let mut iter = bla.enumerate();
    loop {
        match iter.next() {
            Some((i, a)) => {
                arr[a as usize] = i as u32;
            }
            None => break,
        }
    }

    unsafe {
        vec.set_len(len);
        //~^ ERROR: it is unsound to trust return value of `std::iter::ExactSizeIterator::len` and pass it to an unsafe function like `std::vec::Vec::set_len`, which may leak uninitialized memory
        //~[normal]| ERROR: it violates the precondition of `Vec::set_len` to extend a `Vec`'s length without initializing its content in advance
    }
    vec
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
pub fn swap_index_for_enumerate(bla: impl ExactSizeIterator<Item = u32>) -> Vec<u32> {
    let len = bla.len();
    let mut vec = Vec::with_capacity(len); //~[normal] uninit_vec
    let arr: &mut [u32] = unsafe { std::slice::from_raw_parts_mut(vec.as_mut_ptr(), len) };
    //~[normal]^ERROR: it violates the precondition of `std::slice::from_raw_parts_mut` to create a slice from a `Vec` that is not initialized yet
    // `vec` is not initialized
    for (i, a) in bla.enumerate() {
        arr[a as usize] = i as u32;
    }

    unsafe {
        vec.set_len(len);
        //~^ERROR: it is unsound to trust return value of `std::iter::ExactSizeIterator::len` and pass it to an unsafe function like `std::vec::Vec::set_len`, which may leak uninitialized memory
        //~[normal]|ERROR: it violates the precondition of `Vec::set_len` to extend a `Vec`'s length without initializing its content in advance
    }
    vec
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
pub fn set_len(bla: impl ExactSizeIterator<Item = u32>) -> Vec<u32> {
    let mut vec = Vec::new(); //~[normal] uninit_vec
    let len = bla.len();
    // vec.reserve(len);
    // let arr: &mut [u32] = unsafe { std::slice::from_raw_parts_mut(vec.as_mut_ptr(), bla.len()) };
    // for (i, a) in bla.enumerate() {
    //     arr[a as usize] = i as u32;
    // }

    unsafe {
        vec.set_len(len);
        //~^ERROR: it is unsound to trust return value of `std::iter::ExactSizeIterator::len` and pass it to an unsafe function like `std::vec::Vec::set_len`, which may leak uninitialized memory
    }
    vec
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
pub fn clear(bla: impl ExactSizeIterator<Item = u32>) -> Vec<u32> {
    let mut vec = Vec::new();

    vec.clear();
    vec
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
pub fn resize(bla: impl ExactSizeIterator<Item = u32>) -> Vec<u32> {
    let mut vec = Vec::new();
    let len = bla.len();

    vec.resize(len, 0);
    vec
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
pub fn reserve(bla: impl ExactSizeIterator<Item = u32>) -> Vec<u32> {
    let mut vec = Vec::new();
    let len = bla.len();

    vec.reserve(len);
    vec
}

fn main() {}
