//@ignore-on-host
use std::mem::uninitialized;
use std::mem::MaybeUninit;

#[rpl::dump_mir()]
fn uninitialized_value<T>() -> T {
    unsafe { uninitialized() }
}