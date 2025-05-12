use std::mem::transmute;

// produce a value with an invalid state
pub fn invalid_value() -> bool {
    let x: u8 = 10;
    let res = unsafe { transmute::<u8, bool>(x) }; //~ERROR: it is unsound to transmute a type `u8` to a boolean
    res
}

fn main() {
    invalid_value();
}
