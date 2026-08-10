//@rustc-env: RPL_PATS=docs/patterns-pest/safedrop
//@compile-flags: -Z mir-opt-level=0
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
struct Entry<K, V> {
    key: K,
    value: V,
    next: *mut Entry<K, V>,
    prev: *mut Entry<K, V>,
}
struct IntoIter<K, V> {
    head: *mut Entry<K, V>,
    remaining: usize,
    tail: *mut Entry<K, V>,
}
impl<K, V> DoubleEndedIterator for IntoIter<K, V> {
    fn next_back(&mut self) -> Option<(K, V)> {
        if self.remaining == 0 {
            return None;
        }
        self.remaining -= 1;
        unsafe {
            let next = (*self.tail).next;
            let e = *Box::from_raw(self.tail);
            self.tail = next;
            Some((e.key, e.value))
        }
    }
}
impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        self.remaining -= 1;
        unsafe {
            let prev = (*self.head).prev;
            let e = *Box::from_raw(self.head);
            self.head = prev;
            Some((e.key, e.value))
        }
    }
}
impl<K, V> Drop for IntoIter<K, V> {
    fn drop(&mut self) {
        for _ in 0..self.remaining {
            unsafe {
                let next = (*self.tail).next;
                Box::from_raw(self.tail);
                //~^ ERROR: this unsafe operation may use a pointer after its owner has been dropped
                self.tail = next;
            }
        }
    }
}

fn linked_hash_map_next_back() {
    let first = Box::into_raw(Box::new(Entry {
        key: String::from("a"),
        value: String::from("10"),
        next: std::ptr::null_mut(),
        prev: std::ptr::null_mut(),
    }));
    let middle = Box::into_raw(Box::new(Entry {
        key: String::from("b"),
        value: String::from("20"),
        next: std::ptr::null_mut(),
        prev: std::ptr::null_mut(),
    }));
    let last = Box::into_raw(Box::new(Entry {
        key: String::from("c"),
        value: String::from("30"),
        next: std::ptr::null_mut(),
        prev: std::ptr::null_mut(),
    }));
    unsafe {
        (*first).prev = middle;
        (*middle).prev = last;
        (*middle).next = last;
        (*last).next = middle;
    }
    let mut iter = IntoIter {
        head: first,
        remaining: 3,
        tail: last,
    };
    let _ = iter.next();
    let _ = iter.next_back();
}

fn main() {
    linked_hash_map_next_back();
}
