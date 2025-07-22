use std::mem;

pub unsafe fn get_data<T: ?Sized>(val: *const T) -> *const () {
    unsafe { *mem::transmute::<*const *const T, *const *const ()>(&val) }
    //~^ ERROR: wrong assumption of fat pointer layout
}

pub unsafe  fn get_data_mut<T: ?Sized>(mut val: *mut T) -> *mut () {
    unsafe { *mem::transmute::<*mut *mut T, *mut *mut ()>(&mut val) }
    //~^ ERROR: wrong assumption of fat pointer layout
}

fn main() {}
