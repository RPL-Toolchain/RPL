//@rustc-env: RPL_PATS=tests/ui/gaps/memory_alias/memory_alias.rpl
//@check-pass

//! Faithfulness boundary: memory (false negative).
//!
//! When `p` and `q` alias, the read observes the write.  `alpha` keeps places as
//! syntax, so `*p` and `*q` are unrelated and the pattern (which needs one place)
//! is not matched.  After adding a conservative may-alias analysis this test
//! should flip to the diagnostic below.
#![allow(dead_code)]
#![allow(unsafe_op_in_unsafe_fn)]

unsafe fn mem(p: *mut u8, q: *mut u8) -> u8 {
    *p = 1;
    *q
}
