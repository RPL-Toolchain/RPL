//@rustc-env: RPL_PATS=tests/ui/features/session/shared_ty_mismatch_sig.rpl
//@check-pass
//@compile-flags: -Z inline-mir=false
//
// Expected: no match — `$T`/`$U` disagree across the two signature-only functions.

#![allow(dead_code)]

struct Pair<T, U> {
    first: T,
    second: U,
}

fn takes_u8(_: Pair<u8, u16>) {}

fn takes_u16(_: Pair<u16, u8>) {}

fn main() {}
