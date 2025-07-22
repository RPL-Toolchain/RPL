use std::alloc::{alloc, dealloc, Layout};

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn alloc_write<T: Default>() {
    let layout = Layout::new::<T>();
    unsafe {
        let ptr = unsafe { alloc(layout) as *mut T };
        ptr.write(T::default());
        //~^ERROR: it is an undefined behavior to dereference a null pointer, and `std::alloc::alloc` may return a null pointer
        dealloc(ptr as *mut u8, layout)
    }
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn alloc_cast_check_write<T: Default>() {
    let layout = Layout::new::<T>();
    unsafe {
        let ptr = unsafe { alloc(layout) as *mut T };
        assert!(!ptr.is_null());
        ptr.write(T::default());
        dealloc(ptr as *mut u8, layout)
    }
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn alloc_check_cast_write<T: Default>() {
    let layout = Layout::new::<T>();
    unsafe {
        let ptr = unsafe { alloc(layout) };
        assert!(!ptr.is_null());
        let ptr = ptr as *mut T;
        ptr.write(T::default());
        dealloc(ptr as *mut u8, layout)
    }
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn alloc_cast_check_as_write<T: Default>() {
    let layout = Layout::new::<T>();
    unsafe {
        let ptr = unsafe { alloc(layout) as *mut T };
        assert_ne!(ptr as usize, 0);
        ptr.write(T::default());
        dealloc(ptr as *mut u8, layout)
    }
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn alloc_check_as_cast_write<T: Default>() {
    let layout = Layout::new::<T>();
    unsafe {
        let ptr = unsafe { alloc(layout) };
        assert_ne!(ptr as usize, 0);
        let ptr = ptr as *mut T;
        ptr.write(T::default());
        dealloc(ptr as *mut u8, layout)
    }
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn alloc_maybe_misaligned_and_write<T: Default>() {
    let layout = Layout::from_size_align(size_of::<T>(), 8).unwrap();
    unsafe {
        let ptr = unsafe { alloc(layout) as *mut T };
        //FIXME: the alignment may be wrong, try checking this case
        assert!(!ptr.is_null());
        ptr.write(T::default());
        dealloc(ptr as *mut u8, layout)
    }
}

fn main() {
    alloc_write::<usize>();
    alloc_write::<String>();
    alloc_cast_check_write::<usize>();
    alloc_cast_check_write::<String>();
    alloc_check_cast_write::<usize>();
    alloc_check_cast_write::<String>();
    alloc_cast_check_as_write::<usize>();
    alloc_cast_check_as_write::<String>();
    alloc_check_as_cast_write::<usize>();
    alloc_check_as_cast_write::<String>();
    alloc_maybe_misaligned_and_write::<u64>();
    alloc_maybe_misaligned_and_write::<u128>();
}
