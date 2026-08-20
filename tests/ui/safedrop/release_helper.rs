//@rustc-env: RPL_PATS=docs/patterns-safedrop
//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=true -Z inline-mir-threshold=10000
//@ check-pass
#![allow(dead_code, unused_must_use, unsafe_op_in_unsafe_fn)]

use std::mem;
use std::ptr;

struct Node<K, V> {
    key: K,
    value: V,
    next: *mut Node<K, V>,
}

unsafe fn drop_empty_node<K, V>(ptr: *mut Node<K, V>) {
    let Node { key, value, .. } = unsafe { *Box::from_raw(ptr) };
    mem::forget(key);
    mem::forget(value);
}

struct Map<K, V> {
    entries: Vec<K>,
    free: *mut Node<K, V>,
}

impl<K, V> Map<K, V> {
    fn clear_free_list(&mut self) {
        unsafe {
            let mut free = self.free;
            while !free.is_null() {
                let next = (*free).next;
                drop_empty_node(free);
                free = next;
            }
            self.free = ptr::null_mut();
        }
    }

    fn shrink_to_fit(&mut self) {
        self.entries.shrink_to_fit();
        self.clear_free_list();
    }
}

fn main() {
    let mut map: Map<String, String> = Map {
        entries: Vec::new(),
        free: ptr::null_mut(),
    };
    map.shrink_to_fit();
}
