//@rustc-env: RPL_PATS=docs/patterns-safedrop/dangling_pointer.rpl
//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=true

fn return_dangling() -> *const u8 {
    //~^ safedrop_dangling_pointer
    let boxed = Box::new(1u8);
    let ptr = &*boxed as *const u8;
    drop(boxed);
    ptr
}

fn expose_dangling(out: *mut *const u8) {
    //~^ safedrop_dangling_pointer
    let boxed = Box::new(2u8);
    unsafe {
        *out = &*boxed as *const u8;
    }
    drop(boxed);
}

fn return_live(value: &u8) -> *const u8 {
    value as *const u8
}

fn main() {
    let live = 3u8;
    let mut out = core::ptr::null();
    let _ = return_dangling();
    expose_dangling(&mut out);
    let _ = return_live(&live);
}
