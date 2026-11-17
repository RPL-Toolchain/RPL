//@rustc-env: RPL_PATS=tests/ui/features/session/adt_def_unify.rpl
//@check-pass
//@compile-flags: -Z inline-mir=false
//
// Expected: no match — one `$Pair` cannot bind two distinct AdtDefs.

#![allow(dead_code)]

struct PairA<T, U> {
    first: T,
    second: U,
}

struct PairB<T, U> {
    first: T,
    second: U,
}

fn uses_a<T: Copy, U>(p: &PairA<T, U>) -> T {
    p.first
}

fn uses_b<T: Copy, U>(p: &PairB<T, U>) -> T {
    p.first
}

fn main() {}
