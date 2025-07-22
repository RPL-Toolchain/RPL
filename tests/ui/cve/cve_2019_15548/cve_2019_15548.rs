//@compile-flags: -lncurses
//@revisions: inline regular
//@[inline]compile-flags: -Z inline-mir=true
//@[regular]compile-flags: -Z inline-mir=false
use std::mem;

mod ll {
    use libc::{c_char, c_int};

    unsafe extern "C" {
        pub fn instr(_: *const c_char) -> c_int;
    }
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
pub fn instr(s: &mut String) -> i32 {
    /* XXX: This is probably broken. */
    unsafe {
        let buf = s.as_bytes().as_ptr();
        //~^ ERROR: it is usually a bug to cast a `&str` to a `*const libc::c_char`, and then pass it to an extern function
        let ret = ll::instr(mem::transmute(buf));
        //~^ ERROR: it is usually a bug to pass a buffer pointer to an extern function without specifying its length

        let capacity = s.capacity();
        match s.find('\0') {
            Some(index) => s.as_mut_vec().set_len(index as usize),
            None => s.as_mut_vec().set_len(capacity),
        }

        ret
    }
}

fn main() {
    let mut s = String::from("");
    let code = instr(&mut s);
}
