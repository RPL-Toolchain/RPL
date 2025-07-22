//@revisions: inline
//@[inline]compile-flags: -Z inline-mir=true
//@compile-flags: -A unsafe-op-in-unsafe-fn
//@check-pass
//FIXME: This case can't be detected yet, as `Trailer::allocate` is not inlined.
use std::{
    alloc,
    default::Default,
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut, Drop},
    ptr, slice,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Trailer<T> {
    ptr: *mut u8,
    size: usize,
    phantom: PhantomData<T>,
}

impl<T: Default> Trailer<T> {
    // #[rpl::dump_mir(dump_cfg, dump_ddg)]
    pub fn new(capacity: usize) -> Trailer<T> {
        unsafe {
            let trailer = Trailer::allocate(capacity);
            let ptr = trailer.ptr as *mut T;
            ptr.write(T::default());
            trailer
        }
    }
}

impl<T> Trailer<T> {
    unsafe fn allocate(capacity: usize) -> Trailer<T> {
        let size = mem::size_of::<T>() + capacity;
        let align = mem::align_of::<T>();
        let layout = alloc::Layout::from_size_align(size, align).unwrap();
        let ptr = alloc::alloc_zeroed(layout);

        Trailer {
            ptr,
            size,
            phantom: PhantomData,
        }
    }
}
