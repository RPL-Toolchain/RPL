//@rustc-env: RPL_PATS=docs/patterns-safedrop
//@compile-flags: -Z mir-opt-level=1 -Z inline-mir=true
//@ check-pass
#![allow(unsafe_op_in_unsafe_fn)]

struct Slot {
    ptr: *mut u8,
}

impl Slot {
    unsafe fn replace_and_write(&mut self, new_ptr: *mut u8) {
        unsafe { drop(Box::from_raw(self.ptr)) };
        self.ptr = new_ptr;

        let reborrow = &mut *self;
        unsafe { reborrow.ptr.write(3) };
    }
}

struct Buffer {
    array: Box<[u8]>,
    len: usize,
}

impl Buffer {
    fn capacity(&self) -> usize {
        self.array.len()
    }

    fn offset(&self, index: usize) -> usize {
        (self.capacity() + index) % self.capacity()
    }

    fn copy_to(&self, destination: &mut [u8]) {
        destination[..self.array.len()].copy_from_slice(&self.array);
    }

    fn replace_and_read(&mut self) -> usize {
        let mut new_array = vec![0; self.capacity() * 2].into_boxed_slice();
        self.copy_to(&mut new_array);
        self.array = new_array;
        self.offset(self.len)
    }
}

fn main() {
    let old_ptr = Box::into_raw(Box::new(1));
    let new_ptr = Box::into_raw(Box::new(2));
    let mut slot = Slot { ptr: old_ptr };

    unsafe {
        slot.replace_and_write(new_ptr);
        drop(Box::from_raw(slot.ptr));
    }

    let mut buffer = Buffer {
        array: vec![0; 4].into_boxed_slice(),
        len: 1,
    };
    assert_eq!(buffer.replace_and_read(), 1);
}
