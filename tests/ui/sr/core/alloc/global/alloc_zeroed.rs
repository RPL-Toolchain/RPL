//@compile-flags: -Z mir-opt-level=1 -Z inline-mir=false

use std::alloc::{GlobalAlloc, Layout};
use std::ptr;

struct TestAllocator;

unsafe impl GlobalAlloc for TestAllocator {
    unsafe fn alloc(&self, _: Layout) -> *mut u8 {
        ptr::null_mut()
    }

    unsafe fn dealloc(&self, _: *mut u8, _: Layout) {}
}

fn main() {
    unsafe {
        let unchecked = Layout::from_size_align_unchecked(0, 8);
        let _ = GlobalAlloc::alloc_zeroed(&TestAllocator, unchecked);
        //~^ unsafe_numeric_precondition

        let checked = Layout::from_size_align(0, 8).unwrap();
        let _ = GlobalAlloc::alloc_zeroed(&TestAllocator, checked);
        //~^ unsafe_numeric_precondition

        let new = Layout::new::<[u8; 0]>();
        let _ = GlobalAlloc::alloc_zeroed(&TestAllocator, new);
        //~^ unsafe_numeric_precondition

        let array = Layout::array::<u8>(0).unwrap();
        let _ = GlobalAlloc::alloc_zeroed(&TestAllocator, array);
        //~^ unsafe_numeric_precondition

        let nonzero = Layout::from_size_align_unchecked(8, 8);
        let _ = GlobalAlloc::alloc_zeroed(&TestAllocator, nonzero);
    }
}
