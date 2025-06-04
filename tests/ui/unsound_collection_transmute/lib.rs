use std::mem::transmute;

fn f1() {
    unsafe { transmute::<_, Vec<u32>>(vec![2_u16]) };
    //~^ ERROR: transmutes from `Vec<u16>` to `Vec<u32>` where `u16` and `u32` have different ABI, size or alignment
}
