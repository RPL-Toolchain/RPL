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
struct Buffer {
    data: Vec<u8>,
    len: usize,
}
impl Buffer {
    fn allocate(len: usize) -> Self {
        Self {
            data: vec![0; len],
            len,
        }
    }
    fn copy_to(&self, out: &mut Buffer) -> usize {
        let n = self.len.min(out.data.len());
        out.data[..n].copy_from_slice(&self.data[..n]);
        n
    }
    fn as_mut_ptr(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }
    fn len(&self) -> usize {
        self.data.len()
    }
}
impl From<Buffer> for Vec<u8> {
    fn from(buffer: Buffer) -> Vec<u8> {
        //~^ ERROR: this function may expose a dangling pointer
        let mut slice = Buffer::allocate(buffer.len);
        let len = buffer.copy_to(&mut slice);
        unsafe { Vec::from_raw_parts(slice.as_mut_ptr(), len, slice.len()) }
    }
}

fn buffer_into_vec() {
    let buffer = Buffer::allocate(8);
    let bytes: Vec<u8> = buffer.into();
    let _ = bytes.len();
}

fn main() {
    buffer_into_vec();
}
