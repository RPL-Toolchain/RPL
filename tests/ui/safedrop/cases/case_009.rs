//@rustc-env: RPL_PATS=docs/patterns-pest/safedrop
//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=true
#![allow(dead_code, unsafe_op_in_unsafe_fn)]

use std::mem::MaybeUninit;
use std::ptr;

struct SparseChunk<A> {
    map: [bool; 2],
    data: MaybeUninit<[MaybeUninit<A>; 2]>,
}

impl<A> SparseChunk<A> {
    unsafe fn slot(&mut self, index: usize) -> *mut A {
        (*self.data.as_mut_ptr())[index].as_mut_ptr()
    }

    fn pair(index1: usize, value1: A, index2: usize, value2: A) -> Self {
        let mut chunk = Self {
            map: [false; 2],
            data: MaybeUninit::uninit(),
        };

        chunk.map[index1] = true;
        unsafe { ptr::write(chunk.slot(index1), value1) };
        chunk.map[index2] = true;
        unsafe { ptr::write(chunk.slot(index2), value2) };
        chunk
    }
    //~^ ERROR: this unsafe operation may free storage that is already dead
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
    let _ = SparseChunk::pair(0, Box::new(1), 1, Box::new(2));
}
