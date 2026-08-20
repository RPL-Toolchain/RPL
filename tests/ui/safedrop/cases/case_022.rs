//@rustc-env: RPL_PATS=docs/patterns-safedrop
//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=true
#![allow(dead_code, unsafe_op_in_unsafe_fn)]

use std::ffi::c_void;

struct RollbackHook<'a>(&'a String);

impl Copy for RollbackHook<'_> {}

impl Clone for RollbackHook<'_> {
    fn clone(&self) -> Self {
        *self
    }
}

fn main() {
    let data;
    {
        let captured = String::from("rollback");
        let hook = Box::new(RollbackHook(&captured));
        data = (&raw const *hook).cast::<c_void>();
        let _ = Box::into_raw(hook);
    }

    unsafe {
        let hook = *data.cast::<RollbackHook<'_>>();
        //~^ ERROR: this unsafe operation may use a pointer after its owner has been dropped
        std::hint::black_box(hook.0.len());
    }
}
