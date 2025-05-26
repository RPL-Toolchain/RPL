//@revisions: normal
//@[normal]compile-flags: -Z inline-mir=false
use std::{
    alloc::{Layout, alloc_zeroed, dealloc},
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
            //~|NOTE:  See https://doc.rust-lang.org/std/alloc/fn.alloc_zeroed.html and https://doc.rust-lang.org/std/alloc/trait.GlobalAlloc.html#method.alloc_zeroed
            //~|NOTE:  `#[deny(rpl::alloc_maybe_zero)]` on by default
        }

        Self {
            p: ptr as *mut u64,
            sz_u64,
            layout,
        }
    }
}
