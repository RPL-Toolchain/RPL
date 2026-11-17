//@rustc-env: RPL_PATS=tests/ui/features/session/impl_cross_impl.rpl
//@check-pass
//@compile-flags: -Z inline-mir=false
//
// Expected: no match — `a` and `b` live in two different `impl S` blocks
// (`impl_of_method` differs) even though `$S` is the same AdtDef.

#![allow(dead_code)]

struct S<T> {
    first: T,
}

impl<T: Copy> S<T> {
    fn a(&self) -> T {
        self.first
    }
}

impl<T: Copy> S<T> {
    fn b(&self) -> T {
        self.first
    }
}

fn main() {}
