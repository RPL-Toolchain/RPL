//@rustc-env: RPL_PATS=docs/patterns-safedrop
//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=true
#![allow(dead_code, unsafe_op_in_unsafe_fn)]

use std::mem::{self, MaybeUninit};
use std::ptr;

struct SparseChunk<A> {
    map: [bool; 2],
    data: MaybeUninit<[MaybeUninit<A>; 2]>,
}

impl<A> SparseChunk<A> {
    unsafe fn slot(&mut self, index: usize) -> *mut A {
        (*self.data.as_mut_ptr())[index].as_mut_ptr()
    }

    fn unit(index: usize, value: A) -> Self { //~ ERROR: this function may expose a dangling pointer
        let mut chunk = Self {
            map: [false; 2],
            data: MaybeUninit::uninit(),
        };
        if mem::replace(&mut chunk.map[index], true) {
            let old = unsafe { ptr::read(chunk.slot(index)) };
            unsafe { ptr::write(chunk.slot(index), value) };
            drop(old);
        } else { //~ ERROR: this unsafe operation may free storage that is already dead
            unsafe { ptr::write(chunk.slot(index), value) };
        }
        chunk
    }
    //~^ ERROR: this unsafe operation may free storage that is already dead
    //~| ERROR: this unsafe operation may free storage that is already dead
}

impl<A> Drop for SparseChunk<A> {
    fn drop(&mut self) {
        for index in 0..2 {
            if self.map[index] {
                unsafe { ptr::drop_in_place(self.slot(index)) }
            }
        }
    }
}

fn main() {
    let _ = SparseChunk::unit(0, Box::new(1));
}
