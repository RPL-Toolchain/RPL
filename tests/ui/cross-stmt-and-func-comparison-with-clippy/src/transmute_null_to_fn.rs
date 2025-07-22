//@revisions: inline normal
//@[normal] compile-flags: -Zinline-mir=false
#![expect(clippy::zero_ptr)]

#[cfg_attr(test, test)]
fn base_case() {
    unsafe {
        let x: fn() = std::mem::transmute(0 as *const u64);
        //~^ transmute_null_to_fn
        dbg!(x);

        let x: fn() = std::mem::transmute(std::ptr::null::<u64>());
        //~^ transmute_null_to_fn
        dbg!(x);
    }
}

#[cfg_attr(test, test)]
fn cross_function_null_ptr() {
    const fn const_null_ptr<T>() -> *const T {
        std::ptr::null()
    }
    fn null_ptr<T>() -> *const T {
        std::ptr::null()
    }
    unsafe {
        let x: fn() = std::mem::transmute(const_null_ptr::<u64>());
        //~[inline]^ transmute_null_to_fn
        dbg!(x);

        let x: fn() = std::mem::transmute(null_ptr::<u64>());
        //~[inline]^ transmute_null_to_fn
        dbg!(x);
    }
}

#[cfg_attr(test, test)]
fn cross_statement() {
    let null_ptr = 0 as *const u64;
    unsafe {
        let x: fn() = std::mem::transmute(null_ptr);
        //~^ transmute_null_to_fn
        dbg!(x);
    }

    let null_ptr = std::ptr::null::<u64>();
    unsafe {
        let x: fn() = std::mem::transmute(null_ptr);
        //~^ transmute_null_to_fn
        dbg!(x);
    }
}

pub(crate) fn main() {
    base_case();
    cross_function_null_ptr();
    cross_statement();
}
