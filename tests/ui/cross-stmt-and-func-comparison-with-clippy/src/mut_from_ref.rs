//@revisions: inline normal
//@[normal] compile-flags: -Zinline-mir=false
fn base_case(x: &u32) -> &mut u16 {
    //~^ mut_from_ref

    unsafe { &mut *(&raw const *x).cast_mut().cast() }
}

#[cfg_attr(test, test)]
fn base_case_test() {
    let x = 42;
    let y = base_case(&x);
    dbg!(*y);
    *y += 1;
    dbg!(*y);
}

pub(crate) fn main() {
    base_case_test();
}
