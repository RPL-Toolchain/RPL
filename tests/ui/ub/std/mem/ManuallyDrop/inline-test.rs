//@ignore-on-host
use std::mem::ManuallyDrop;

#[rpl::dump_mir(dump_cfg, dump_ddg)]
fn double_drop_string() {
    let mut s = ManuallyDrop::new("1".to_owned());
    unsafe {
        ManuallyDrop::drop(&mut s);
        ManuallyDrop::drop(&mut s);
    }
}

#[rpl::dump_mir(dump_cfg, dump_ddg)]
fn double_drop<T>(value: T) {
    let mut s = ManuallyDrop::new(value);
    unsafe {
        ManuallyDrop::drop(&mut s);
        ManuallyDrop::drop(&mut s);
    }
}

fn main() {
    double_drop_string();
    double_drop("1".to_owned());
}
