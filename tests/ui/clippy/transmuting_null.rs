//@revisions: inline normal
//@[inline] compile-flags: -Z mir-opt-level=0 -Z inline-mir=true
//@[inline] ignore-on-host
//@[normal] compile-flags: -Z mir-opt-level=0 -Z inline-mir=false
#![allow(dead_code)]
#![allow(clippy::zero_ptr)]
#![allow(clippy::transmute_ptr_to_ref)]
#![allow(clippy::eq_op, clippy::missing_transmute_annotations)]
#![allow(clippy::manual_dangling_ptr)]

// Easy to lint because these only span one line.
// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn one_liners() {
    unsafe {
        let _: &u64 = std::mem::transmute(0 as *const u64);
        //~^ transmuting_null

        let _: &u64 = std::mem::transmute(std::ptr::null::<u64>());
        //~^ transmuting_null
    }
}

pub const ZPTR: *const usize = 0 as *const _;
pub const NOT_ZPTR: *const usize = 1 as *const _;

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn transmute_const() {
    unsafe {
        // Should raise a lint.
        let _: &u64 = std::mem::transmute(ZPTR);
        //~^ transmuting_null

        // Should NOT raise a lint.
        let _: &u64 = std::mem::transmute(NOT_ZPTR);
    }
}

fn main() {
    one_liners();
    transmute_const();
}
