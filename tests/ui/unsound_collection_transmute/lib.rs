use std::mem::transmute;

fn f1() {
    unsafe { transmute::<_, Vec<u32>>(vec![2_u16]) };
    //~^ ERROR: transmute from `std::vec::Vec<u16>` to `std::vec::Vec<u32>` with mismatched layout is unsound
}
