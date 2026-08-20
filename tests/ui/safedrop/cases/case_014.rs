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
use std::{ptr, slice};

#[derive(Debug, PartialEq)]
struct SliceDeque<T> {
    data: Vec<T>,
}
impl<T> SliceDeque<T> {
    fn new() -> Self {
        Self { data: Vec::new() }
    }
    fn push_back(&mut self, value: T) {
        self.data.push(value);
    }
    fn len(&self) -> usize {
        self.data.len()
    }
    fn drain_filter<F>(&mut self, pred: F) -> DrainFilter<'_, T, F>
    where
        F: FnMut(&mut T) -> bool,
    {
        let old_len = self.data.len();
        unsafe {
            self.data.set_len(0);
        }
        DrainFilter {
            deque: self,
            pred,
            idx: 0,
            del: 0,
            old_len,
        }
    }
}

struct DrainFilter<'a, T, F>
where
    F: FnMut(&mut T) -> bool,
{
    deque: &'a mut SliceDeque<T>,
    pred: F,
    idx: usize,
    del: usize,
    old_len: usize,
}
impl<'a, T, F> Iterator for DrainFilter<'a, T, F>
where
    F: FnMut(&mut T) -> bool,
{
    type Item = T;
    fn next(&mut self) -> Option<T> {
        unsafe {
            while self.idx != self.old_len {
                let i = self.idx;
                self.idx += 1;
                let v = slice::from_raw_parts_mut(self.deque.data.as_mut_ptr(), self.old_len);
                if (self.pred)(&mut v[i]) {
                    self.del += 1;
                    return Some(ptr::read(&v[i]));
                } else if self.del > 0 {
                    let del = self.del;
                    let src: *const T = &v[i];
                    let dst: *mut T = &mut v[i - del];
                    ptr::copy_nonoverlapping(src, dst, 1);
                }
            }
        }
        None
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.old_len - self.idx))
    }
}
impl<'a, T, F> Drop for DrainFilter<'a, T, F>
where
    F: FnMut(&mut T) -> bool,
{
    fn drop(&mut self) {
        for _ in self.by_ref() {}
        //~^ ERROR: this unsafe operation may free storage that is already dead
        unsafe {
            self.deque.data.set_len(self.old_len - self.del);
        }
    }
}

macro_rules! sdeq {
    () => {
        SliceDeque::new()
    };
    [] => {
        SliceDeque::new()
    };
    [$($x:expr),* $(,)?] => {{
        let mut d = SliceDeque::new();
        $(d.push_back($x);)*
        d
    }};
}

fn drain_filter_predicate_panics() {
    let mut deq = SliceDeque::new();
    deq.push_back(Box::new(1));
    deq.push_back(Box::new(2));
    let mut iter = deq.drain_filter(|_| panic!("predicate failed"));
    let _ = iter.next();
}
//~^ ERROR: this unsafe operation may free storage that is already dead

fn main() {
    drain_filter_predicate_panics();
}
