//@revisions: normal
//@[normal]compile-flags: -Z inline-mir=false
use std::alloc::{Layout, alloc, alloc_zeroed, dealloc};

pub fn alloc_maybe_zero(sz_u64: usize) {
    let sz_bytes = sz_u64 * size_of::<u64>();
    let layout = Layout::from_size_align(sz_bytes, 8).unwrap();
    let ptr = unsafe { alloc(layout) };
    //~^ERROR: public function `alloc_maybe_zero` allocates a pointer that may be zero-sized, which is an undefined behavior
    unsafe { dealloc(ptr, layout) }
}

pub fn alloc_zeroed_maybe_zero(sz_u64: usize) {
    let sz_bytes = sz_u64 * size_of::<u64>();
    let layout = Layout::from_size_align(sz_bytes, 8).unwrap();
    let ptr = unsafe { alloc_zeroed(layout) };
    //~^ERROR: public function `alloc_zeroed_maybe_zero` allocates a pointer that may be zero-sized, which is an undefined behavior
    unsafe { dealloc(ptr, layout) }
}
