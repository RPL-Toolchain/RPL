//@revisions: normal
//@[normal] compile-flags: -Z inline-mir=false

//FIXME: there are a lot of false positives in this test
use std::mem::ManuallyDrop;

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn double_drop() {
    let mut s = ManuallyDrop::new("1".to_owned());
    unsafe {
        ManuallyDrop::drop(&mut s);
        //~^ERROR: invalid sequence of operations on `core::mem::ManuallyDrop`: `drop` and `drop`
        ManuallyDrop::drop(&mut s);
        //~^ERROR: invalid sequence of operations on `core::mem::ManuallyDrop`: `drop` and `drop`
    }
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn double_take() {
    let mut s = ManuallyDrop::new("1".to_owned());
    unsafe {
        let t1 = ManuallyDrop::take(&mut s);
        //~^ERROR: invalid sequence of operations on `core::mem::ManuallyDrop`: `take` and `take`
        let t2 = ManuallyDrop::take(&mut s);
        //~^ERROR: invalid sequence of operations on `core::mem::ManuallyDrop`: `take` and `take`
    }
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn drop_after_take() {
    let mut s = ManuallyDrop::new("1".to_owned());
    unsafe {
        let t = ManuallyDrop::take(&mut s);
        //~^ERROR: invalid sequence of operations on `core::mem::ManuallyDrop`: `drop` and `take`
        ManuallyDrop::drop(&mut s);
        //~^ERROR: invalid sequence of operations on `core::mem::ManuallyDrop`: `take` and `drop`
    }
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn take_after_drop() {
    let mut s = ManuallyDrop::new("1".to_owned());
    unsafe {
        ManuallyDrop::drop(&mut s);
        //~^ERROR: invalid sequence of operations on `core::mem::ManuallyDrop`: `take` and `drop`
        let t = ManuallyDrop::take(&mut s);
        //~^ERROR: invalid sequence of operations on `core::mem::ManuallyDrop`: `drop` and `take`
    }
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn into_inner_after_drop() {
    let mut s = ManuallyDrop::new("1".to_owned());
    unsafe {
        ManuallyDrop::drop(&mut s);
        let t2 = ManuallyDrop::into_inner(s);
        //~^ERROR: invalid sequence of operations on `core::mem::ManuallyDrop`: `drop` and `into_inner`
    }
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn into_inner_after_take() {
    let mut s = ManuallyDrop::new("1".to_owned());
    unsafe {
        let t1 = ManuallyDrop::take(&mut s);
        let t2 = ManuallyDrop::into_inner(s);
        //~^ERROR: invalid sequence of operations on `core::mem::ManuallyDrop`: `take` and `into_inner`
    }
}

fn single_drop() {
    let mut s = ManuallyDrop::new("1".to_owned());
    unsafe {
        ManuallyDrop::drop(&mut s);
    }
}

fn drop_in_loop() {
    let mut s = ManuallyDrop::new("1".to_owned());
    for _ in 0..10 {
        unsafe {
            ManuallyDrop::drop(&mut s);
            //FIXME: detect this
        }
    }
}

fn main() {
    double_drop();
    drop_after_take();
    take_after_drop();
    double_take();
    drop_in_loop();
}
