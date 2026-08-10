//@compile-flags: -Z mir-opt-level=1 -Z inline-mir=false

#![feature(allocator_api)]

use std::alloc::{AllocError, Allocator, Layout};
use std::ptr::NonNull;

struct TestAllocator;

unsafe impl Allocator for TestAllocator {
    fn allocate(&self, _: Layout) -> Result<NonNull<[u8]>, AllocError> {
        Err(AllocError)
    }

    unsafe fn deallocate(&self, _: NonNull<u8>, _: Layout) {}
}

unsafe fn unknown_sizes(old_size: usize, new_size: usize) {
    let align = 8usize;
    let old_layout = unsafe { Layout::from_size_align_unchecked(old_size, align) };
    let new_layout = unsafe { Layout::from_size_align_unchecked(new_size, align) };
    let ptr = NonNull::dangling();
    let _ = unsafe { Allocator::grow_zeroed(&TestAllocator, ptr, old_layout, new_layout) };
}

fn layout_from_align_and_size(align: usize, size: usize) -> Layout {
    unsafe { Layout::from_size_align_unchecked(size, align) }
}

fn main() {
    let ptr = NonNull::dangling();
    let align = 8usize;

    unsafe {
        let old_size = 16usize;
        let too_small = 8usize;
        let old_layout = Layout::from_size_align_unchecked(old_size, align);
        let new_layout = Layout::from_size_align_unchecked(too_small, align);
        let _ = Allocator::grow_zeroed(&TestAllocator, ptr, old_layout, new_layout);
        //~^ unsafe_numeric_precondition

        let equal_size = 16usize;
        let old_layout = Layout::from_size_align_unchecked(old_size, align);
        let new_layout = Layout::from_size_align_unchecked(equal_size, align);
        let _ = Allocator::grow_zeroed(&TestAllocator, ptr, old_layout, new_layout);

        let larger_size = 32usize;
        let old_layout = Layout::from_size_align_unchecked(old_size, align);
        let new_layout = Layout::from_size_align_unchecked(larger_size, align);
        let _ = Allocator::grow_zeroed(&TestAllocator, ptr, old_layout, new_layout);

        let old_layout = layout_from_align_and_size(16, 8);
        let new_layout = layout_from_align_and_size(8, 16);
        let _ = Allocator::grow_zeroed(&TestAllocator, ptr, old_layout, new_layout);

        unknown_sizes(old_size, larger_size);
    }
}
