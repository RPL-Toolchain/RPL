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
    unsafe fn slot(&self, index: usize) -> *const A {
        (*self.data.as_ptr())[index].as_ptr()
    }

    unsafe fn slot_mut(&mut self, index: usize) -> *mut A {
        (*self.data.as_mut_ptr())[index].as_mut_ptr()
    }
}

impl<A: Clone> Clone for SparseChunk<A> {
    fn clone(&self) -> Self {
        let mut out = Self {
            map: [false; 2],
            data: MaybeUninit::uninit(),
        };
        for index in 0..2 {
            if self.map[index] {
                let value = unsafe { (&*self.slot(index)).clone() };
                out.map[index] = true;
                unsafe { ptr::write(out.slot_mut(index), value) };
            }
        }
        out
    }
    //~^ ERROR: this unsafe operation may free storage that is already dead
}

impl<A> Drop for SparseChunk<A> {
    fn drop(&mut self) {
        for index in 0..2 {
            if self.map[index] {
                unsafe { ptr::drop_in_place(self.slot_mut(index)) }
            }
        }
    }
}

fn main() {}
