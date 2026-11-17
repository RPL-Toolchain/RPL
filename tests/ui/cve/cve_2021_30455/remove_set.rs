//@compile-flags: -Z inline-mir=false

//! CVE-2021-30457-shaped: `get_unchecked_mut` (weak) then `drop_in_place`.

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
        if self.values.len() <= id {
            self.values.reserve(id + 1 - self.values.len());
            //~^ uninit_vec
            // false positive above
            unsafe {
                self.values.set_len(id + 1);
            }
        }
        unsafe {
            ptr::write(self.values.as_mut_ptr().add(id), val);
        }
        self.occupied[id] = true;
    }

    fn remove_set(&mut self, to_remove: &[bool]) {
        let n = self.occupied.len().min(to_remove.len());
        for id in 0..n {
            if self.occupied[id] && to_remove[id] {
                unsafe {
                    ptr::drop_in_place(self.values.get_unchecked_mut(id));
                    //~^ ERROR: weak lifetime-bypassing operation reaches potentially panicking / unresolvable generic code
                }
                self.occupied[id] = false;
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
    let mut map = IdMap::new();
    map.insert(0, String::from("x"));
    map.remove_set(&[true]);
}
