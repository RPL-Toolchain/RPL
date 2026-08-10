//@rustc-env: RPL_PATS=docs/patterns-pest/safedrop
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
use std::mem;
use std::ops::Range;
use std::ptr;
trait Array {
    type Item;
}
struct SmallVec<A: Array> {
    data: Vec<A::Item>,
}
impl<A: Array> SmallVec<A> {
    fn len(&self) -> usize {
        self.data.len()
    }
    fn extend<I: IntoIterator<Item = A::Item>>(&mut self, iter: I) {
        self.data.extend(iter);
    }
    fn reserve(&mut self, n: usize) {
        self.data.reserve(n);
    }
    fn as_mut_ptr(&mut self) -> *mut A::Item {
        self.data.as_mut_ptr()
    }
    unsafe fn set_len(&mut self, len: usize) {
        self.data.set_len(len);
    }
    //~v ERROR: this function may expose a dangling pointer
    pub fn insert_many<I: IntoIterator<Item = A::Item>>(&mut self, index: usize, iterable: I) {
        let iter = iterable.into_iter();
        if index == self.len() {
            return self.extend(iter);
        }
        let (lower_size_bound, _) = iter.size_hint();
        assert!(lower_size_bound <= core::isize::MAX as usize);
        assert!(index + lower_size_bound >= index);
        self.reserve(lower_size_bound);
        unsafe {
            let old_len = self.len();
            assert!(index <= old_len);
            let start = self.as_mut_ptr();
            let mut ptr = start.add(index);
            ptr::copy(ptr, ptr.add(lower_size_bound), old_len - index);
            self.set_len(0);
            let mut guard = DropOnPanic {
                start,
                skip: index..(index + lower_size_bound),
                len: old_len + lower_size_bound,
            };
            let mut num_added = 0;
            for element in iter {
                let mut cur = ptr.add(num_added);
                if num_added >= lower_size_bound {
                    self.reserve(1);
                    let start = self.as_mut_ptr();
                    ptr = start.add(index);
                    cur = ptr.add(num_added);
                    ptr::copy(cur, cur.add(1), old_len - index);
                    guard.start = start;
                    guard.len += 1;
                    guard.skip.end += 1;
                }
                ptr::write(cur, element);
                guard.skip.start += 1;
                num_added += 1;
            }
            mem::forget(guard);
            if num_added < lower_size_bound {
                ptr::copy(
                    ptr.add(lower_size_bound),
                    ptr.add(num_added),
                    old_len - index,
                );
            }
            self.set_len(old_len + num_added);
        }
        //~^ ERROR: this unsafe operation may free storage that is already dead
        struct DropOnPanic<T> {
            start: *mut T,
            skip: Range<usize>,
            len: usize,
        }
        impl<T> Drop for DropOnPanic<T> {
            fn drop(&mut self) {
                for i in 0..self.len {
                    if !self.skip.contains(&i) {
                        unsafe {
                            ptr::drop_in_place(self.start.add(i));
                        }
                    }
                }
            }
        }
    }
    //~^ ERROR: this unsafe operation may free storage that is already dead
}

struct DropDetector(u32);

impl DropDetector {
    fn new(num: u32) -> Self {
        DropDetector(num)
    }
}

impl Drop for DropDetector {
    fn drop(&mut self) {
        let _ = self.0;
    }
}

struct DropArray;

impl Array for DropArray {
    type Item = DropDetector;
}

struct PanickingIterator {
    current: u32,
    panic_at: u32,
    len: usize,
}

impl Iterator for PanickingIterator {
    type Item = DropDetector;
    fn next(&mut self) -> Option<Self::Item> {
        let num = self.current;
        if num == self.panic_at {
            panic!("panicking index");
        }
        self.current += 1;
        Some(DropDetector::new(num))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl ExactSizeIterator for PanickingIterator {}

fn insert_many_panic() {
    let mut vec = SmallVec::<DropArray> {
        data: vec![
            DropDetector::new(1),
            DropDetector::new(2),
            DropDetector::new(3),
        ],
    };
    vec.insert_many(
        1,
        PanickingIterator {
            current: 1,
            panic_at: 1,
            len: 1,
        },
    );
}

fn main() {
    insert_many_panic();
}
