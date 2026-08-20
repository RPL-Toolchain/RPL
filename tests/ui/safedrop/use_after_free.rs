//@rustc-env: RPL_PATS=docs/patterns-safedrop/use_after_free.rpl
//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=true
fn main() {
    unsafe {
        let read_ptr: *const u8;
        {
            let boxed = Box::new(1u8);
            read_ptr = &*boxed as *const u8;
        }
        let _ = core::ptr::read(read_ptr);
        //~^ safedrop_use_after_free

        let write_ptr: *mut u8;
        {
            let mut boxed = Box::new(2u8);
            write_ptr = &mut *boxed as *mut u8;
        }
        core::ptr::write(write_ptr, 3);
        //~^ safedrop_use_after_free

        let read_volatile_ptr: *const u8;
        {
            let boxed = Box::new(5u8);
            read_volatile_ptr = &*boxed as *const u8;
        }
        let _ = core::ptr::read_volatile(read_volatile_ptr);
        //~^ safedrop_use_after_free

        let write_volatile_ptr: *mut u8;
        {
            let mut boxed = Box::new(6u8);
            write_volatile_ptr = &mut *boxed as *mut u8;
        }
        core::ptr::write_volatile(write_volatile_ptr, 7);
        //~^ safedrop_use_after_free

        let copy_src_ptr: *const u8;
        let mut copy_dst = 0u8;
        {
            let boxed = Box::new(8u8);
            copy_src_ptr = &*boxed as *const u8;
        }
        core::ptr::copy(copy_src_ptr, &mut copy_dst as *mut u8, 1);
        //~^ safedrop_use_after_free

        let copy_dst_ptr: *mut u8;
        let copy_src = 9u8;
        {
            let mut boxed = Box::new(10u8);
            copy_dst_ptr = &mut *boxed as *mut u8;
        }
        core::ptr::copy(&copy_src as *const u8, copy_dst_ptr, 1);
        //~^ safedrop_use_after_free

        let copy_nonoverlapping_src_ptr: *const u8;
        let mut copy_nonoverlapping_dst = 0u8;
        {
            let boxed = Box::new(11u8);
            copy_nonoverlapping_src_ptr = &*boxed as *const u8;
        }
        core::ptr::copy_nonoverlapping(
            //~^ safedrop_use_after_free
            copy_nonoverlapping_src_ptr,
            &mut copy_nonoverlapping_dst as *mut u8,
            1,
        );

        let copy_nonoverlapping_dst_ptr: *mut u8;
        let copy_nonoverlapping_src = 12u8;
        {
            let mut boxed = Box::new(13u8);
            copy_nonoverlapping_dst_ptr = &mut *boxed as *mut u8;
        }
        core::ptr::copy_nonoverlapping(
            //~^ safedrop_use_after_free
            &copy_nonoverlapping_src as *const u8,
            copy_nonoverlapping_dst_ptr,
            1,
        );

        let write_bytes_ptr: *mut u8;
        {
            let mut boxed = Box::new(14u8);
            write_bytes_ptr = &mut *boxed as *mut u8;
        }
        core::ptr::write_bytes(write_bytes_ptr, 0, 1);
        //~^ safedrop_use_after_free

        let live = Box::new(4u8);
        let live_ptr = &*live as *const u8;
        let _ = core::ptr::read(live_ptr);
    }
}
