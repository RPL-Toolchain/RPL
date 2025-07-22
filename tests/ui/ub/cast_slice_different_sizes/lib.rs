use std::mem::MaybeUninit;

pub fn f1() {
    let a = [1_u8, 2, 3, 4];
    let p = &a as *const [u8] as *const [u32];
    //~^ cast_slice_different_sizes
    unsafe {
        println!("{:?}", &*p);
    }
}

pub fn f2() {
    let a = [1_u8, 2, 3, 4];
    let p = &a as *const [u8] as *const [u8];
    unsafe {
        println!("{:?}", &*p);
    }
}

pub fn f3() {
    let a = [1_u8, 2, 3, 4];
    let p = &a as *const [u8] as *const [MaybeUninit<u8>];
    unsafe {
        println!("{:?}", &*p);
    }
}
