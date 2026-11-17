//@compile-flags: -Z inline-mir=false

//! CVE-2021-30456-shaped: `get_unchecked_mut` (weak) then user `F`.

#![allow(dead_code)]

use std::ptr;

struct IdMap<T> {
    occupied: Vec<bool>,
    values: Vec<T>,
}

impl<T> IdMap<T> {
    fn with_capacity(cap: usize) -> Self {
        let mut values = Vec::with_capacity(cap);
        //~^ uninit_vec
        // false positive above
        unsafe {
            values.set_len(cap);
            //~^ set_len_uninitialized
        }
        Self {
            occupied: vec![false; cap],
            values,
        }
    }

    fn get_or_insert_with<F>(&mut self, id: usize, f: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        if !self.occupied[id] {
            self.occupied[id] = true;
            unsafe {
                let space = self.values.get_unchecked_mut(id);
                ptr::write(space, f());
                //~^ ERROR: weak lifetime-bypassing operation reaches potentially panicking / unresolvable generic code
            }
        }
        unsafe { self.values.get_unchecked_mut(id) }
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
    let mut map = IdMap::<String>::with_capacity(1);
    let _ = map.get_or_insert_with(0, || String::from("x"));
}
