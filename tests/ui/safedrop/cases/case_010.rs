//@rustc-env: RPL_PATS=docs/patterns-safedrop
//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=true
#![allow(dead_code, unsafe_op_in_unsafe_fn)]

use std::ops::Index;
use std::sync::{Arc, Mutex};

struct Mapping<T> {
    values: Vec<T>,
}

impl<T> Mapping<T> {
    fn ptr(&self) -> *const T {
        self.values.as_ptr()
    }
}

#[derive(Clone)]
struct Buffer<T> {
    data: Arc<Mutex<Mapping<T>>>,
}

impl<T> Buffer<T> {
    fn new(values: Vec<T>) -> Self {
        Self {
            data: Arc::new(Mutex::new(Mapping { values })),
        }
    }

    fn replace_mapping(&self, values: Vec<T>) {
        *self.data.lock().unwrap() = Mapping { values };
        //~^ ERROR: this unsafe operation may free storage that is already dead
    }
}

impl<T> Index<usize> for Buffer<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        unsafe {
            let mut count = index;
            let mut ptr = self.data.lock().unwrap().ptr();
            while count > 0 {
                count -= 1;
                ptr = ptr.offset(1);
            }
            &*ptr
        }
    }
}

fn reference_outlives_mapping_lock() {
    let buffer = Buffer::new(vec![1, 2, 3]);
    let resizer = buffer.clone();
    let value = &buffer[1];
    resizer.replace_mapping(vec![4]);
    std::hint::black_box(value);
}
//~^ ERROR: this unsafe operation may free storage that is already dead

fn main() {
    reference_outlives_mapping_lock();
}
