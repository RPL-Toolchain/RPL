//@rustc-env: RPL_PATS=docs/patterns-pest/safedrop
//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=true
#![allow(dead_code, unsafe_op_in_unsafe_fn)]

use std::mem::MaybeUninit;

fn main() {
    let mut array: [MaybeUninit<Box<u8>>; 3] = std::array::from_fn(|_| MaybeUninit::uninit());
    array[0].write(Box::new(1));
    array[1].write(Box::new(2));
    array[2].write(Box::new(3));

    let mut start = 0;
    let mut length = 3;

    let front = unsafe { array[start].assume_init_read() };
    start = (start + 1) % 3;
    length -= 1;
    drop(front);

    // Vulnerable pop_back uses length - 1 instead of (start + length - 1) % capacity.
    let back = unsafe { array[length - 1].assume_init_read() };
    //~^ ERROR: this unsafe operation may use a pointer after its owner has been dropped
    length -= 1;
    drop(back);
    //~^ ERROR: this unsafe operation may free storage that is already dead

    let back = unsafe { array[length - 1].assume_init_read() };
    //~^ ERROR: this unsafe operation may use a pointer after its owner has been dropped
    drop(back);
    //~^ ERROR: this unsafe operation may free storage that is already dead
    std::hint::black_box(start);
}
