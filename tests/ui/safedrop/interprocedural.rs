//@rustc-env: RPL_PATS=docs/patterns-safedrop
//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=true
#![allow(dead_code, unsafe_op_in_unsafe_fn)]

use std::alloc::{Layout, alloc, dealloc};

#[inline(never)]
unsafe fn inspect(ptr: *const String) -> usize {
    unsafe { (*ptr).len() }
}

#[inline(never)]
unsafe fn overwrite(ptr: *mut usize) {
    unsafe { *ptr = 1 }
}

#[inline(never)]
fn ignore(_: *const String) {}

struct Handle(*const String);

#[inline(never)]
unsafe fn consume(handle: Handle) -> usize {
    unsafe { (*handle.0).len() }
}

#[inline(never)]
unsafe fn dispose(ptr: *mut u8, layout: Layout) {
    unsafe { dealloc(ptr, layout) }
}

#[inline(never)]
unsafe fn dispose_wrapper(ptr: *mut u8, layout: Layout) {
    unsafe { dispose(ptr, layout) }
}

fn main() {
    let dangling;
    {
        let owner = Box::new(String::from("used by callee"));
        dangling = &*owner as *const String;
    }
    unsafe {
        let _ = inspect(dangling);
        //~^ ERROR: this unsafe operation may use a pointer after its owner has been dropped
    }

    let ignored;
    {
        let owner = Box::new(String::from("not used by callee"));
        ignored = &*owner as *const String;
    }
    ignore(ignored);

    let moved;
    {
        let owner = Box::new(String::from("moved into callee"));
        moved = Handle(&*owner);
    }
    unsafe {
        let _ = consume(moved);
        //~^ ERROR: this unsafe operation may use a pointer after its owner has been dropped
    }

    let writable;
    {
        let mut owner = Box::new(0usize);
        writable = &mut *owner as *mut usize;
    }
    unsafe {
        overwrite(writable);
        //~^ ERROR: this unsafe operation may use a pointer after its owner has been dropped
    }

    unsafe {
        let layout = Layout::new::<u8>();
        let ptr = alloc(layout);
        dispose_wrapper(ptr, layout);
        ptr.write(1);
        //~^ ERROR: this unsafe operation may use a pointer after its owner has been dropped
    }
}
