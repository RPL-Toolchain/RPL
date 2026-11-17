//@rustc-env: RPL_PATS=tests/ui/features/inlined_call_smallvec/inlined_call_smallvec.rpl

//! `SmallVec::set_len` combines a const generic (`[u8; 4]`) with an inlined body that keeps a
//! call (`triple_mut`) plus a write, so one call yields several reports.

#![allow(dead_code)]

extern crate smallvec;
use smallvec::SmallVec;

#[inline(never)]
fn set_len_called(v: &mut SmallVec<[u8; 4]>) {
    unsafe {
        v.set_len(1);
        //~^ ERROR: inlined `SmallVec::set_len` call detected
        //~| ERROR: inlined `SmallVec::set_len` call detected
        //~| ERROR: inlined `SmallVec::set_len` call detected
    }
}

fn main() {
    let mut v: SmallVec<[u8; 4]> = SmallVec::new();
    set_len_called(&mut v);
}
