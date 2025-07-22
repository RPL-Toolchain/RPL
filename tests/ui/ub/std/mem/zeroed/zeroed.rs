//@ignore-on-host

#[rpl::dump_mir()]
fn zeroed_initialized_reference<T>() -> &'static T {
    let x: &'static T = unsafe { std::mem::zeroed() };
    x
}

#[rpl::dump_mir()]
fn zeroed_initialized_reference_mut<T>() -> &'static mut T {
    let x: &'static mut T = unsafe { std::mem::zeroed() };
    x
}

fn main() {}
