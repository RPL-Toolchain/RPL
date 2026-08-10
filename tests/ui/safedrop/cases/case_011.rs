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
use std::mem::ManuallyDrop;
use std::vec::Vec as StdVec;
struct CompactVec<T> {
    ptr: *mut T,
    len: usize,
    cap: usize,
}
impl<T> CompactVec<T> {
    fn parts(&self) -> (usize, usize) {
        (self.len, self.cap)
    }
    fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr
    }
    fn from_stdvec_unchecked(mut v: StdVec<T>) -> Self {
        let out = Self {
            ptr: v.as_mut_ptr(),
            len: v.len(),
            cap: v.capacity(),
        };
        std::mem::forget(v);
        out
    }
    //~v ERROR: this function may expose a dangling pointer
    fn with<'a, R: 'a, F: FnOnce(&mut StdVec<T>) -> R>(&mut self, f: F) -> R {
        let (len, cap) = self.parts();
        let mut stdvec = unsafe { StdVec::from_raw_parts(self.as_mut_ptr(), len, cap) };
        let r = f(&mut stdvec);
        ManuallyDrop::new(core::mem::replace(
            self,
            Self::from_stdvec_unchecked(stdvec),
        ));
        r
    }
}

fn panic_in_client_code() {
    let mut compact = CompactVec::from_stdvec_unchecked(vec![1, 2, 3]);
    compact.with(|_| {
        panic!("client code panicked");
    });
}

fn main() {
    panic_in_client_code();
}
