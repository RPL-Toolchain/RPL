//@ revisions: inline regular
//@[inline] compile-flags: -Z inline-mir=true
//@[regular] compile-flags: -Z inline-mir=false

extern crate libc;

use libc::{sockaddr, sockaddr_storage, socklen_t};
use std::mem::{self, MaybeUninit};
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
use std::ptr;

struct Addr {
    storage: sockaddr_storage,
    len: socklen_t,
}

impl Addr {
    /// Creates a raw socket address from `SocketAddr`.
    fn new(addr: std::net::SocketAddr) -> Self {
        match &addr {
            SocketAddr::V4(addr) => Self::new_v4(addr),
            SocketAddr::V6(addr) => Self::new_v6(addr),
        }
    }

    fn new_v4(addr: &std::net::SocketAddrV4) -> Self {
        let (addr, len): (*const sockaddr, socklen_t) = (
            addr as *const SocketAddrV4 as *const sockaddr,
            //~^ ERROR: wrong assumption of layout compatibility from `std::net::SocketAddrV4` to `libc::sockaddr`
            mem::size_of_val(addr) as socklen_t,
        );
        unsafe { Self::from_raw_parts(addr, len) }
    }

    // #[rpl::dump_mir(dump_cfg, dump_ddg)]
    fn new_v6(addr: &std::net::SocketAddrV6) -> Self {
        let (addr, len): (*const sockaddr, socklen_t) = (
            addr as *const SocketAddrV6 as *const sockaddr,
            //~^ ERROR: wrong assumption of layout compatibility from `std::net::SocketAddrV6` to `libc::sockaddr`
            mem::size_of_val(addr) as socklen_t,
        );
        unsafe { Self::from_raw_parts(addr, len) }
    }

    /// Creates an `Addr` from its raw parts.
    unsafe fn from_raw_parts(addr: *const sockaddr, len: socklen_t) -> Self {
        let mut storage = MaybeUninit::<sockaddr_storage>::uninit();
        unsafe {
            ptr::copy_nonoverlapping(
                addr as *const u8,
                &mut storage as *mut MaybeUninit<sockaddr_storage> as *mut u8,
                len as usize,
            );
        }
        Self {
            storage: unsafe { storage.assume_init() },
            len,
        }
    }
}

fn main() {
    todo!()
}
