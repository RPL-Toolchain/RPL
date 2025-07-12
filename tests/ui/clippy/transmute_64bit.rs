//@revisions: inline normal
//@ignore-bitwidth: 32
//@[inline]compile-flags: -Z mir-opt-level=0 -Z inline-mir=true
//@[normal]compile-flags: -Z mir-opt-level=0 -Z inline-mir=false

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn main() {
    unsafe {
        let _: *const usize = std::mem::transmute(6.0f64);
        //~^ wrong_transmute

        let _: *mut usize = std::mem::transmute(6.0f64);
        //~^ wrong_transmute
    }
}
