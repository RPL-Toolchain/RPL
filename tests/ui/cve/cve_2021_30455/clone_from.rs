//@compile-flags: -Z inline-mir=false

//! CVE-2021-30455-shaped: `ptr::read` then unresolvable `clone`.

#![allow(dead_code)]

use std::ptr;

struct IdMap<T> {
    occupied: Vec<bool>,
    values: Vec<T>,
}

impl<T> IdMap<T> {
    fn new() -> Self {
        Self {
            occupied: Vec::new(),
            values: Vec::new(),
        }
    }

    fn insert(&mut self, id: usize, val: T) {
        if id >= self.occupied.len() {
            self.occupied.resize(id + 1, false);
        }
        if self.values.capacity() < id + 1 {
            self.values.reserve(id + 1);
        }
        unsafe {
            ptr::write(self.values.as_mut_ptr().add(id), val);
        }
        self.occupied[id] = true;
    }
}

impl<T: Clone> Clone for IdMap<T> {
    fn clone(&self) -> Self {
        let mut out = IdMap::new();
        out.clone_from(self);
        out
    }

    fn clone_from(&mut self, other: &Self) {
        for (id, live) in self.occupied.iter().enumerate() {
            if *live {
                unsafe {
                    ptr::drop_in_place(self.values.as_mut_ptr().add(id));
                }
            }
        }
        self.occupied = other.occupied.clone();
        let cap = other.values.capacity();
        self.values.reserve(cap);
        unsafe {
            for (id, live) in self.occupied.iter().enumerate() {
                if *live {
                    let src = ptr::read(other.values.as_ptr().add(id));
                    let cloned = (<T as Clone>::clone)(&src);
                    //~^ ERROR: lifetime-bypassing operation reaches potentially panicking / unresolvable generic code
                    //~| ERROR: weak lifetime-bypassing operation reaches potentially panicking / unresolvable generic code
                    ptr::write(self.values.as_mut_ptr().add(id), cloned);
                    std::mem::forget(src);
                }
            }
        }
    }
}

impl<T> Drop for IdMap<T> {
    fn drop(&mut self) {
        for (id, live) in self.occupied.iter().enumerate() {
            if *live {
                unsafe {
                    ptr::drop_in_place(self.values.as_mut_ptr().add(id));
                }
            }
        }
    }
}

fn main() {
    let mut a = IdMap::new();
    a.insert(0, String::from("a"));
    let mut b = IdMap::new();
    b.insert(0, String::from("b"));
    b.clone_from(&a);
}
