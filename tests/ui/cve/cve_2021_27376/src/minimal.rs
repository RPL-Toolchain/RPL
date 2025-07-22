fn main() {
    let s: *const core::net::SocketAddrV4 = core::ptr::null();
    let t1 = s as *const libc::sockaddr;
    //~^ ERROR: wrong assumption of layout compatibility from `std::net::SocketAddrV4` to `libc::sockaddr`
    let s: *const core::net::SocketAddrV6 = core::ptr::null();
    let t2 = s as *const libc::sockaddr;
    //~^ ERROR: wrong assumption of layout compatibility from `std::net::SocketAddrV6` to `libc::sockaddr`
}
