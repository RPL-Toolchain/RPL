//@rustc-env: RPL_PATS=docs/patterns-pest/safedrop
//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=true
#![allow(dead_code, unsafe_op_in_unsafe_fn)]

use std::ffi::c_void;

struct Collation<'a>(&'a String);

impl Copy for Collation<'_> {}

impl Clone for Collation<'_> {
    fn clone(&self) -> Self {
        *self
    }
}

fn main() {
    let data;
    {
        let captured = String::from("collation");
        let callback = Box::new(Collation(&captured));
        data = (&raw const *callback).cast::<c_void>();
        let _ = Box::into_raw(callback);
    }

    unsafe {
        let callback = *data.cast::<Collation<'_>>();
        //~^ ERROR: this unsafe operation may use a pointer after its owner has been dropped
        std::hint::black_box((callback.0.len(), "left".cmp("right")));
    }
}
