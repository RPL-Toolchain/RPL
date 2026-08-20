//@rustc-env: RPL_PATS=docs/patterns-safedrop
//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=true
#![allow(dead_code, unsafe_op_in_unsafe_fn)]

fn main() {
    let shared;
    {
        let owner = Box::new(1usize);
        shared = &*owner as *const usize;
    }
    unsafe {
        let _ = &*shared;
        //~^ ERROR: this unsafe operation may use a pointer after its owner has been dropped
    }

    let mutable;
    {
        let mut owner = Box::new(2usize);
        mutable = &mut *owner as *mut usize;
    }
    unsafe {
        let _ = &mut *mutable;
        //~^ ERROR: this unsafe operation may use a pointer after its owner has been dropped
    }
}
