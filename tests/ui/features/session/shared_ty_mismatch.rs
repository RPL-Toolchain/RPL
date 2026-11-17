//@rustc-env: RPL_PATS=tests/ui/features/session/shared_ty_mismatch.rpl
//@check-pass
//@compile-flags: -Z inline-mir=false
//
// Expected: no match — `$S`/`$T` disagree across the two MIR matches.

#![allow(dead_code)]

struct Pair<T, U> {
    first: T,
    second: U,
}

fn takes_u8(p: &Pair<u8, u16>) -> u8 {
    p.first
}

fn takes_u16(p: &Pair<u16, u8>) -> u16 {
    p.first
}

fn main() {}
