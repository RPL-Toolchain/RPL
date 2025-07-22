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
    //~[regular]| ERROR: public function `new_from_template` allocates a pointer that may be zero-sized, which is an undefined behavior
    assert!(!ptr.is_null());
    for i in 0..size {
        unsafe {
            ptr.write(template.clone());
        }
    }
    unsafe { dealloc(ptr as *mut u8, layout) }
}

#[derive(Clone, Copy)]
#[repr(align(32))]
pub struct Align32([u8; 32]);

pub fn new_from_template_32_8(size: usize, template: &Align32) {
    let objsize = std::mem::size_of::<Align32>();
    let layout = Layout::from_size_align(size * objsize, 8).unwrap();
    let ptr = unsafe { alloc(layout) as *mut Align32 };
    //~[regular]^ ERROR: resulting pointer `*mut Align32` has a different alignment than the original alignment that the pointer was created with
    //~[regular]| ERROR: public function `new_from_template_32_8` allocates a pointer that may be zero-sized, which is an undefined behavior
    assert!(!ptr.is_null());
    for i in 0..size {
        unsafe {
            ptr.write(template.clone());
        }
    }
    unsafe { dealloc(ptr as *mut u8, layout) }
}

pub fn new_from_template_8_1(size: usize, template: &u64) {
    let objsize = std::mem::size_of::<u64>();
    let layout = Layout::from_size_align(size * objsize, 1).unwrap();
    let ptr = unsafe { alloc(layout) as *mut u64 };
    //~[regular]^ ERROR: resulting pointer `*mut u64` has a different alignment than the original alignment that the pointer was created with
    //~[regular]| ERROR: public function `new_from_template_8_1` allocates a pointer that may be zero-sized, which is an undefined behavior
    assert!(!ptr.is_null());
    for i in 0..size {
        unsafe {
            ptr.write(template.clone());
        }
    }
    unsafe { dealloc(ptr as *mut u8, layout) }
}

pub fn new_from_template_8_8(size: usize, template: &u64) {
    let objsize = std::mem::size_of::<u64>();
    let layout = Layout::from_size_align(size * objsize, 8).unwrap();
    let ptr = unsafe { alloc(layout) as *mut u64 };
    //~[regular]^ ERROR: public function `new_from_template_8_8` allocates a pointer that may be zero-sized, which is an undefined behavior
    assert!(!ptr.is_null());
    for i in 0..size {
        unsafe {
            ptr.write(template.clone());
        }
    }
    unsafe { dealloc(ptr as *mut u8, layout) }
}

pub fn new_from_template_1_8(size: usize, template: &u8) {
    let objsize = std::mem::size_of::<u8>();
    let layout = Layout::from_size_align(size * objsize, 8).unwrap();
    let ptr = unsafe { alloc(layout) };
    //~[regular]^ ERROR: public function `new_from_template_1_8` allocates a pointer that may be zero-sized, which is an undefined behavior
    assert!(!ptr.is_null());
    for i in 0..size {
        unsafe {
            ptr.write(template.clone());
        }
    }
    unsafe { dealloc(ptr, layout) }
}
