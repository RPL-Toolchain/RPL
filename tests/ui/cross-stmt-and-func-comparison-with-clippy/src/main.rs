//@ignore-on-host: check individual modules instead
#![feature(rustc_attrs)]
#![feature(register_tool)]
#![register_tool(rpl)]
#![deny(clippy::correctness)]
#![allow(internal_features)]
mod cast_slice_different_sizes;
mod eager_transmute;
mod mem_replace_with_uninit;
mod mut_from_ref;
mod not_unsafe_ptr_arg_deref;
mod size_of_in_element_count;
mod transmute_collection;
mod transmute_null_to_fn;
mod transmuting_null;
mod uninit_assumed_init;
mod uninit_vec;
mod zero_offset;

fn main() {
    cast_slice_different_sizes::main();
    eager_transmute::main();
    mem_replace_with_uninit::main();
    mut_from_ref::main();
    not_unsafe_ptr_arg_deref::main();
    size_of_in_element_count::main();
    transmute_collection::main();
    transmuting_null::main();
    transmute_null_to_fn::main();
    uninit_assumed_init::main();
    uninit_vec::main();
    zero_offset::main();
}
