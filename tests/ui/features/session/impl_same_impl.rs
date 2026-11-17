//@rustc-env: RPL_PATS=tests/ui/features/session/impl_same_impl.rpl
//@compile-flags: -Z inline-mir=false
//
// Expected: both methods of the same `impl` match. Two ERROR annotations per site
// because `$a`/`$b` slot permutations are distinct SessionResults.

#![allow(dead_code)]

struct S<T> {
    first: T,
}

impl<T: Copy> S<T> {
    fn a(&self) -> T {
        self.first
        //~^ ERROR: session impl methods matched on the same impl
        //~| ERROR: session impl methods matched on the same impl
    }
    fn b(&self) -> T {
        self.first
        //~^ ERROR: session impl methods matched on the same impl
        //~| ERROR: session impl methods matched on the same impl
    }
}

fn main() {}
