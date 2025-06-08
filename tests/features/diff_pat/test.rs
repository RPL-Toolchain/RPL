fn main() {
    let x = &[1, 2, 3];
    let y = x as *const i32;
    let z = unsafe { y.offset(1) };
    //~^ diff_pat
    println!("{:?}", unsafe { *z });
}

fn f1(len: usize) {
    assert!(len < 2usize);
    let x = &[1, 2, 3];
    let y = x as *const i32;
    let z = unsafe { y.offset(1) };
    println!("{:?}", unsafe { *z });
}

fn f2(len: usize) {
    assert!(2usize > len);
    let x = &[1, 2, 3];
    let y = x as *const i32;
    let z = unsafe { y.offset(1) };
    println!("{:?}", unsafe { *z });
}
