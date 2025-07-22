// https://doc.rust-lang.org/std/mem/fn.transmute.html#transmutation-between-pointers-and-integers
// Transmuting integers to pointers is a largely unspecified operation.
// It is likely not equivalent to an as cast.
// Doing non-zero-sized memory accesses with a pointer constructed this way
// is currently considered undefined behavior.
use std::mem::transmute;

pub fn transmute_usize_to_ptr(x: usize) {
    //~^ERROR: it is unsound to transmute an integer type `usize` to a pointer type `*const ()`
    let ptr: *const () = unsafe { transmute(x) };
    let ptr_usize = ptr as *const usize;
    println!("{}", unsafe { *ptr_usize });
}

pub fn transmute_isize_to_ptr(x: isize) {
    //~^ERROR: it is unsound to transmute an integer type `isize` to a pointer type `*const ()`
    let ptr: *const () = unsafe { transmute(x) };
    let ptr_isize = ptr as *const isize;
    println!("{}", unsafe { *ptr_isize });
}

pub fn transmute_u64_to_mut_ptr(x: u64) {
    //~^ERROR: it is unsound to transmute an integer type `u64` to a pointer type `*mut ()`
    let ptr: *mut () = unsafe { transmute(x) };
    let ptr_u64 = ptr as *mut u64;
    println!("{}", unsafe { *ptr_u64 });
}

struct S {
    s: u64,
}

impl S {
    pub fn transmute_u64_to_mut_ptr(x: u64) -> *mut u64 {
        //~^ERROR: it is unsound to transmute an integer type `u64` to a pointer type `*mut ()`
        let ptr: *mut () = unsafe { transmute(x) };
        ptr as *mut u64
    }
}

fn main() {}
