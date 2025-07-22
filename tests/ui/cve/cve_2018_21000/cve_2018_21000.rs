use std::mem::{forget, size_of};
use std::vec::Vec;

//#[rpl::dump_mir(dump_cfg, dump_ddg)]
pub unsafe fn guarded_transmute_vec_permissive<T>(mut bytes: Vec<u8>) -> Vec<T> {
    // PermissiveGuard::check::<T>(&bytes).unwrap();
    let ptr = bytes.as_mut_ptr();
    let capacity = bytes.capacity() / size_of::<T>();
    let len = bytes.len() / size_of::<T>();
    forget(bytes);
    unsafe { Vec::from_raw_parts(ptr as *mut T, capacity, len) }
    //~^ ERROR: misordered parameters `len` and `cap` in `Vec::from_raw_parts`
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
pub unsafe fn guarded_transmute_to_bytes_vec<T>(mut from: Vec<T>) -> Vec<u8> {
    let capacity = from.capacity() * size_of::<T>();
    let ptr = from.as_mut_ptr();
    let len = from.len() * size_of::<T>();
    forget(from);
    unsafe { Vec::from_raw_parts(ptr as *mut u8, capacity, len) }
    //~^ ERROR: misordered parameters `len` and `cap` in `Vec::from_raw_parts`
}

fn main() {}
