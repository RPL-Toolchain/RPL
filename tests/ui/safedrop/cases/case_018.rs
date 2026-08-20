//@rustc-env: RPL_PATS=docs/patterns-safedrop
//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=true
#![allow(dead_code, unsafe_op_in_unsafe_fn)]

use std::ffi::c_void;

struct Scalar<'a>(&'a String);

impl Copy for Scalar<'_> {}

impl Clone for Scalar<'_> {
    fn clone(&self) -> Self {
        *self
    }
}

fn main() {
    let data;
    {
        let captured = String::from("scalar");
        let callback = Box::new(Scalar(&captured));
        data = (&raw const *callback).cast::<c_void>();
        let _ = Box::into_raw(callback);
    }

    unsafe {
        let callback = *data.cast::<Scalar<'_>>();
        //~^ ERROR: this unsafe operation may use a pointer after its owner has been dropped
        std::hint::black_box(callback.0.len());
    }
}
