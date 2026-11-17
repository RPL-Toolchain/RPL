//@rustc-env: RPL_PATS=tests/ui/features/session/constraint_unbound.rpl
//@check-pass
//@compile-flags: -Z inline-mir=false
//
// Expected: no match — `$f1` constraints mention `$U` before `$f2` binds it.

#![allow(dead_code)]

struct Pair<T, U> {
    first: T,
    second: U,
}

fn uses_first<T: Copy, U>(p: &Pair<T, U>) -> T {
    p.first
}

fn uses_second<T, U: Copy>(p: &Pair<T, U>) -> U {
    p.second
}

fn main() {}
