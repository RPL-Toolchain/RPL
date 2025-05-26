//@revisions: normal
//@[normal]compile-flags: -Z inline-mir=false
use std::{
    alloc::{alloc_zeroed, dealloc, Layout},
    mem::size_of,
    ops::{Index, IndexMut},
    slice::{from_raw_parts, from_raw_parts_mut},
};

pub struct AlignedMemory<const ALIGN: usize> {
    p: *mut u64,
    sz_u64: usize,
    layout: Layout,
}

impl<const ALIGN: usize> AlignedMemory<{ ALIGN }> {
    pub fn new(sz_u64: usize) -> Self {
        let sz_bytes = sz_u64 * size_of::<u64>();
        let layout = Layout::from_size_align(sz_bytes, ALIGN).unwrap();

        let ptr;
        unsafe {
            ptr = alloc_zeroed(layout);
            //~^ERROR: public function `new` allocates a pointer that may be zero-sized, which is an undefined behavior
        }

        Self {
            p: ptr as *mut u64,
            sz_u64,
            layout,
        }
    }
}
