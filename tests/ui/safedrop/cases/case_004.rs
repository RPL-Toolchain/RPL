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
use std::mem;
use std::ptr;

struct InlineData<T> {
    ptr: *mut T,
}
impl<T> InlineData<T> {
    fn ptr_mut(&mut self) -> *mut T {
        self.ptr
    }
}

enum SmallVecData<T> {
    Inline(InlineData<T>),
    Heap(*mut T),
}
impl<T> SmallVecData<T> {
    fn from_inline(data: InlineData<T>) -> Self {
        Self::Inline(data)
    }
    fn from_heap(ptr: *mut T, _len: usize) -> Self {
        Self::Heap(ptr)
    }
    fn inline_mut(&mut self) -> &mut InlineData<T> {
        match self {
            Self::Inline(data) => data,
            Self::Heap(ptr) => {
                *self = Self::Inline(InlineData { ptr: *ptr });
                match self {
                    Self::Inline(data) => data,
                    _ => unreachable!(),
                }
            }
        }
    }
    fn ptr(&self) -> *mut T {
        match self {
            Self::Inline(data) => data.ptr,
            Self::Heap(ptr) => *ptr,
        }
    }
}

struct SmallVec<T> {
    data: SmallVecData<T>,
    len: usize,
    capacity: usize,
    inline_size_value: usize,
}
impl<T> SmallVec<T> {
    fn inline_size(&self) -> usize {
        self.inline_size_value
    }
    fn spilled(&self) -> bool {
        self.capacity > self.inline_size_value
    }
    unsafe fn triple_mut(&mut self) -> (*mut T, &mut usize, usize) {
        let len = &mut self.len as *mut usize;
        (self.data.ptr(), &mut *len, self.capacity)
    }
    pub fn grow(&mut self, new_cap: usize) {
        //~^ ERROR: this function may expose a dangling pointer
        unsafe {
            let (ptr, &mut len, cap) = self.triple_mut();
            let unspilled = !self.spilled();
            assert!(new_cap >= len);
            if new_cap <= self.inline_size() {
                if unspilled {
                    return;
                }
                self.data = SmallVecData::from_inline(InlineData { ptr });
                ptr::copy_nonoverlapping(ptr, self.data.inline_mut().ptr_mut(), len);
            } else if new_cap != cap {
                let mut vec = Vec::with_capacity(new_cap);
                let new_alloc = vec.as_mut_ptr();
                mem::forget(vec);
                ptr::copy_nonoverlapping(ptr, new_alloc, len);
                self.data = SmallVecData::from_heap(new_alloc, len);
                self.capacity = new_cap;
                if unspilled {
                    return;
                }
            }
            if cap != 0 && !ptr.is_null() {
                let _ = Vec::from_raw_parts(ptr, 0, cap);
            }
        }
    }
}

impl SmallVec<u8> {
    fn new_with_inline(inline_size: usize) -> Self {
        let mut inline = Vec::with_capacity(inline_size);
        let ptr = inline.as_mut_ptr();
        mem::forget(inline);
        Self {
            data: SmallVecData::from_inline(InlineData { ptr }),
            len: 0,
            capacity: inline_size,
            inline_size_value: inline_size,
        }
    }
    fn push(&mut self, value: u8) {
        if self.len == self.capacity {
            self.grow((self.capacity * 2).max(1));
        }
        unsafe {
            ptr::write(self.data.ptr().add(self.len), value);
        }
        self.len += 1;
    }
    fn clear(&mut self) {
        self.len = 0;
    }
    fn len(&self) -> usize {
        self.len
    }
    fn capacity(&self) -> usize {
        self.capacity
    }
}

impl<T> std::fmt::Debug for SmallVec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SmallVec")
            .field("len", &self.len)
            .field("capacity", &self.capacity)
            .finish()
    }
}

fn grow_shrink_to_inline() {
    let mut v = SmallVec::new_with_inline(4);
    v.push(1);
    v.push(2);
    v.push(3);
    v.push(4);
    v.push(5);
    assert!(v.spilled());
    v.clear();
    v.grow(2);
    println!("after grow {:?} len={} cap={}", v, v.len(), v.capacity());
}

fn main() {
    grow_shrink_to_inline();
}
