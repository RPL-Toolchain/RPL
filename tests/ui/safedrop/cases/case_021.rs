//@rustc-env: RPL_PATS=docs/patterns-safedrop
//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=true
#![allow(dead_code, unsafe_op_in_unsafe_fn)]

use std::ffi::c_void;

struct CommitHook<'a>(&'a String);

impl Copy for CommitHook<'_> {}

impl Clone for CommitHook<'_> {
    fn clone(&self) -> Self {
        *self
    }
}

fn main() {
    let data;
    {
        let captured = String::from("commit");
        let hook = Box::new(CommitHook(&captured));
        data = (&raw const *hook).cast::<c_void>();
        let _ = Box::into_raw(hook);
    }

    unsafe {
        let hook = *data.cast::<CommitHook<'_>>();
        //~^ ERROR: this unsafe operation may use a pointer after its owner has been dropped
        std::hint::black_box(!hook.0.is_empty());
    }
}
