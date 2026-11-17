//@rustc-env: RPL_PATS=tests/ui/features/session/required_adt_missing.rpl
//@check-pass
//@compile-flags: -Z inline-mir=false
//
// Expected: no match — required `$Pair` has no AdtDef (only tuples here).

#![allow(dead_code)]

fn uses_a<T: Copy, U>(p: &(T, U)) -> T {
    p.0
}

fn uses_b<T: Copy, U>(p: &(T, U)) -> T {
    p.0
}

fn main() {}
