//@ revisions: inline regular
//@[inline] compile-flags: -Z inline-mir=true
//@[inline] check-pass
//@[regular] compile-flags: -Z inline-mir=false
use std::alloc::{Layout, alloc, dealloc};

pub fn new_from_template<T: Clone>(size: usize, template: &T) {
    let objsize = std::mem::size_of::<T>();
    let layout = Layout::from_size_align(size * objsize, 8).unwrap();
    let ptr = unsafe { alloc(layout) as *mut T };
    //~[regular]^ ERROR: resulting pointer `*mut T` has a different alignment than the original alignment that the pointer was created with
    assert!(!ptr.is_null());
    for i in 0..size {
        unsafe {
            ptr.write(template.clone());
        }
    }
    unsafe { dealloc(ptr as *mut u8, layout) }
}
