//@ignore-on-host
use std::mem::transmute;

fn f1() {
    unsafe { transmute::<_, Vec<u32>>(vec![2_u16]) };
}
