extern crate libc;

use std::ops::Index;
use std::ptr;

pub struct Slab<T> {
    capacity: usize,
    len: usize,
    mem: *mut T,
}

impl<T> Drop for Slab<T> {
    fn drop(&mut self) {
        for x in 0..self.len {
            unsafe {
                let elem_ptr = self.mem.offset(x as isize);
                ptr::drop_in_place(elem_ptr);
                std::hint::black_box(elem_ptr);
            }
        }
        unsafe { libc::free(self.mem as *mut _ as *mut libc::c_void) };
    }
}

impl<T> Index<usize> for Slab<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &(*(self.mem.offset(index as isize))) }
        //~^ERROR: it is an undefined behavior to offset a pointer using an unchecked integer
        //~|HELP:  check whether it's in bound before offsetting
        //~|HELP:  to override `-D warnings` add `#[allow(rpl::unchecked_pointer_offset)]`
    }
}

impl<T> Slab<T> {
    #[inline]
    pub fn remove(&mut self, offset: usize) -> T {
        //~^ ERROR: it usually isn't necessary to apply #[inline] to generic functions
        //~| HELP: See https://matklad.github.io/2021/07/09/inline-in-rust.html and https://rustc-dev-guide.rust-lang.org/backend/monomorph.html
        //~| HELP: to override `-D warnings` add `#[allow(rpl::generic_function_marked_inline)]`
        assert!(offset < self.len, "Offset out of bounds");

        let elem: T;
        let last_elem: T;
        let elem_ptr: *mut T;
        let last_elem_ptr: *mut T;

        unsafe {
            elem_ptr = self.mem.offset(offset as isize);
            //~^ ERROR: it is an undefined behavior to offset a pointer using an unchecked integer
            //~| HELP: check whether it's in bound before offsetting
            last_elem_ptr = self.mem.offset(self.len as isize);
            //~^ HELP: this is because `self.len` exceeds the container's length by one
            //~| HELP: did you mean this

            elem = ptr::read(elem_ptr);
            last_elem = ptr::read(last_elem_ptr);
            //~^ ERROR: pointer out of bound

            ptr::write(elem_ptr, last_elem);
        }

        self.len -= 1;
        return elem;
    }
}
