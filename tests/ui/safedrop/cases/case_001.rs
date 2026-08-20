//@rustc-env: RPL_PATS=docs/patterns-safedrop
//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=true
#![allow(
    dead_code,
    unused_imports,
    unused_variables,
    unused_mut,
    deprecated,
    invalid_value,
    non_camel_case_types,
    unsafe_op_in_unsafe_fn
)]
use std::marker::PhantomData;

struct Buffer<T> {
    ptr: *mut T,
    cap: usize,
    _marker: PhantomData<T>,
}

impl<T> Buffer<T> {
    unsafe fn from_raw_parts(ptr: *mut T, cap: usize) -> Self {
        Self {
            ptr,
            cap,
            _marker: PhantomData,
        }
    }
}

impl<T> Drop for Buffer<T> {
    fn drop(&mut self) {
        if self.cap != 0 && !self.ptr.is_null() {
            unsafe {
                let _ = Vec::from_raw_parts(self.ptr, 0, self.cap);
            }
        }
    }
}

struct SliceDeque<T> {
    head_: usize,
    tail_: usize,
    buf: Buffer<T>,
}

impl<T> SliceDeque<T> {
    fn new() -> Self {
        unsafe { Self::from_raw_parts(std::ptr::null_mut(), 0, 0, 0) }
    }
    fn len(&self) -> usize {
        self.tail_.saturating_sub(self.head_)
    }
    #[allow(rpl::safedrop_dangling_pointer)]
    fn push_back(&mut self, value: T) {
        let len = self.len() + 1;
        let mut data = Vec::with_capacity(len.max(1));
        data.push(value);
        let ptr = data.as_mut_ptr();
        let capacity = data.capacity();
        std::mem::forget(data);
        *self = unsafe { Self::from_raw_parts(ptr, capacity, 0, len) };
    }
    fn pop_front(&mut self) -> Option<T> {
        if self.len() == 0 {
            None
        } else {
            self.head_ += 1;
            None
        }
    }
    fn tail(&self) -> usize {
        self.tail_
    }
    fn head(&self) -> usize {
        self.head_
    }
    fn tail_upper_bound(&self) -> usize {
        self.buf.cap
    }
    fn head_upper_bound(&self) -> usize {
        self.buf.cap
    }

    pub unsafe fn from_raw_parts(ptr: *mut T, capacity: usize, head: usize, tail: usize) -> Self {
        //~^ ERROR: this function may expose a dangling pointer
        debug_assert!(head <= tail);
        let d = Self {
            head_: head,
            tail_: tail,
            buf: Buffer::from_raw_parts(ptr, capacity * 2),
        };
        debug_assert!(d.tail() <= d.tail_upper_bound());
        debug_assert!(d.head() <= d.head_upper_bound());
        d
    }
}

fn main() {
    let mut deque = SliceDeque::new();
    for _ in 0..1_000_000 {
        deque.push_back(String::from("test"));
        if deque.len() == 8 {
            let _ = deque.pop_front();
        }
    }
}
