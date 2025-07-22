//@revisions: normal inline
//@[normal]compile-flags: -Z inline-mir=false
//@[inline]compile-flags: -Z inline-mir=true
//@[inline]check-pass
use std::alloc::{Layout, alloc, dealloc, realloc};

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn use_after_realloc_u8() {
    let layout = Layout::array::<u8>(16).unwrap();
    unsafe {
        let ptr = alloc(layout) as *mut u8;
        assert!(!ptr.is_null());
        ptr.write(42);
        let new_ptr = realloc(ptr as *mut u8, layout, layout.size() * 2) as *mut u8;
        assert!(!new_ptr.is_null());
        // use the old pointer after realloc
        let x = *ptr;
        assert_eq!(x, 42);
        dealloc(ptr as *mut u8, layout);
    }
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn use_after_realloc_read_u32() {
    let layout = Layout::array::<u32>(16).unwrap();
    unsafe {
        let ptr = alloc(layout) as *mut u32;
        assert!(!ptr.is_null());
        ptr.write(42);
        let new_ptr = realloc(ptr as *mut u8, layout, layout.size() * 2) as *mut u32;
        assert!(!new_ptr.is_null());
        // use the old pointer after realloc
        let x = *ptr;
        //~[normal]^ERROR: use a pointer from `u32` after it's reallocated
        assert_eq!(x, 42);
        dealloc(ptr as *mut u8, layout);
    }
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn use_after_realloc_write_u32() {
    let layout = Layout::array::<u32>(16).unwrap();
    unsafe {
        let ptr = alloc(layout) as *mut u32;
        assert!(!ptr.is_null());
        ptr.write(42);
        let new_ptr = realloc(ptr as *mut u8, layout, layout.size() * 2) as *mut u32;
        assert!(!new_ptr.is_null());
        // use the old pointer after realloc
        *ptr = 0;
        //~[normal]^ERROR: use a pointer from `u32` after it's reallocated
        dealloc(ptr as *mut u8, layout);
    }
}

// #[rpl::dump_mir(dump_cfg, dump_ddg)]
fn use_after_realloc_deref_u32() {
    let layout = Layout::array::<u32>(16).unwrap();
    unsafe {
        let ptr = alloc(layout) as *mut u32;
        assert!(!ptr.is_null());
        ptr.write(42);
        let new_ptr = realloc(ptr as *mut u8, layout, layout.size() * 2) as *mut u32;
        assert!(!new_ptr.is_null());
        // use the old pointer after realloc
        let v = &*ptr; //FIXME: false negative
        let v = &mut *ptr;
        //~[normal]^ERROR: use a pointer from `u32` after it's reallocated
        assert_eq!(v, &42);
        dealloc(ptr as *mut u8, layout);
    }
}
