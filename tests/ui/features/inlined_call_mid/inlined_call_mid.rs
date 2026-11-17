//@rustc-env: RPL_PATS=tests/ui/features/inlined_call_mid/inlined_call_mid.rpl

//! Inlined `Vec::set_len` in the middle of a function, followed by more statements: exercises the
//! CFG "collapse" (the inlined call continues in the same block, then more statements follow).
//! One call yields several matches — one per surviving inlined statement — hence three reports.

#![allow(dead_code)]

#[inline(never)]
fn foo(v: &mut Vec<u8>) -> usize {
    unsafe {
        v.set_len(1);
        //~^ ERROR: inlined `Vec::set_len` mid-function detected
        //~| ERROR: inlined `Vec::set_len` mid-function detected
        //~| ERROR: inlined `Vec::set_len` mid-function detected
    }
    let n = v.len();
    n
}

fn main() {
    let mut v = vec![1u8, 2, 3];
    let _ = foo(&mut v);
}
