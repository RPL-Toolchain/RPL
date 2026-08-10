//@rustc-env: RPL_PATS=docs/patterns-pest/safedrop
//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=true
#![allow(dead_code, unsafe_op_in_unsafe_fn)]

use std::ffi::c_void;

struct AggregateStep<'a>(&'a String);

impl Copy for AggregateStep<'_> {}

impl Clone for AggregateStep<'_> {
    fn clone(&self) -> Self {
        *self
    }
}

fn main() {
    let data;
    {
        let captured = String::from("aggregate step");
        let aggregate = Box::new(AggregateStep(&captured));
        data = (&raw const *aggregate).cast::<c_void>();
        let _ = Box::into_raw(aggregate);
    }

    unsafe {
        let aggregate = *data.cast::<AggregateStep<'_>>();
        //~^ ERROR: this unsafe operation may use a pointer after its owner has been dropped
        std::hint::black_box(aggregate.0.len());
    }
}
