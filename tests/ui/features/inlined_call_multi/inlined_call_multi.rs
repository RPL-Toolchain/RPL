//@rustc-env: RPL_PATS=tests/ui/features/inlined_call_multi/inlined_call_multi.rpl
//@revisions: inline regular
//@[inline]compile-flags: -Z inline-mir=true
//@[regular]compile-flags: -Z inline-mir=false

//! Two inlined `Vec::set_len` calls in one function: the matcher should find both.

#![allow(dead_code)]

#[inline(never)]
fn foo(v: &mut Vec<u8>) {
    unsafe {
        v.set_len(1);
        //~^ inlined_set_len_multi
        v.set_len(2);
        //~^ inlined_set_len_multi
    }
}

fn main() {
    let mut v = vec![1u8, 2, 3];
    foo(&mut v);
}
