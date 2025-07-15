//@revisions: inline normal
//@[inline]compile-flags: -Z mir-opt-level=0 -Z inline-mir=true
//@[normal]compile-flags: -Z mir-opt-level=0 -Z inline-mir=false
#![allow(dead_code)]

// Easy to lint because these only span one line.
fn one_liners() {
    unsafe {
        let _: fn() = std::mem::transmute(0 as *const ());
        //~^ transmute_null_to_fn

        let _: fn() = std::mem::transmute(std::ptr::null::<()>());
        //~^ transmute_null_to_fn
    }
}

pub const ZPTR: *const usize = 0 as *const _;
pub const NOT_ZPTR: *const usize = 1 as *const _;

fn transmute_const() {
    unsafe {
        // Should raise a lint.
        let _: fn() = std::mem::transmute(ZPTR);
        //~^ transmute_null_to_fn

        // Should NOT raise a lint.
        let _: fn() = std::mem::transmute(NOT_ZPTR);
    }
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn issue_11485() {
    unsafe {
        let _: fn() = std::mem::transmute(0 as *const u8 as *const ());
        //~^ transmute_null_to_fn

        let _: fn() = std::mem::transmute(std::ptr::null::<()>() as *const u8);
        //~^ transmute_null_to_fn

        let _: fn() = std::mem::transmute(ZPTR as *const u8);
        //~^ transmute_null_to_fn
    }
}

fn main() {
    one_liners();
    //~[inline]^ transmute_null_to_fn
    //~[inline]| transmute_null_to_fn
    transmute_const();
    //~[inline]^ transmute_null_to_fn
}
