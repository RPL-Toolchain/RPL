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
                //~^ ptr_offset_with_cast
                //~| HELP: if you’re always increasing the pointer address, you can avoid the numeric cast by using the `add` method instead.
                //~| HELP: to override `-D warnings` add `#[allow(rpl::ptr_offset_with_cast)]`
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
        assert!(index < self.len, "Index out of bounds");
        unsafe { &(*(self.mem.offset(index as isize))) }
        //~^ERROR: it is an undefined behavior to offset a pointer using an unchecked integer
        //~| HELP:  check whether it's in bound before offsetting
        //~| HELP:  to override `-D warnings` add `#[allow(rpl::unchecked_pointer_offset_general)]`
        //~| ptr_offset_with_cast
        //~| HELP: if you’re always increasing the pointer address, you can avoid the numeric cast by using the `add` method instead.
    }
}

impl<T> Slab<T> {
    unsafe fn mut_elem_ptr(&mut self, offset: usize) -> *mut T {
        unsafe { self.mem.offset(offset as isize) }
    }
    unsafe fn last_mut_elem_ptr(&mut self) -> *mut T {
        unsafe { self.mut_elem_ptr(self.len - 1) }
    }
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
            elem_ptr = self.mut_elem_ptr(offset);
            //~^ ERROR: it is an undefined behavior to offset a pointer using an unchecked integer
            //~| HELP: check whether it's in bound before offsetting
            //~| ptr_offset_with_cast
            //~| HELP: if you’re always increasing the pointer address, you can avoid the numeric cast by using the `add` method instead.
            last_elem_ptr = self.last_mut_elem_ptr();
            //~^ ptr_offset_with_cast
            //~| HELP: if you’re always increasing the pointer address, you can avoid the numeric cast by using the `add` method instead.

            elem = ptr::read(elem_ptr);
            last_elem = ptr::read(last_elem_ptr);

            ptr::write(elem_ptr, last_elem);
        }

        self.len -= 1;
        return elem;
    }
}
