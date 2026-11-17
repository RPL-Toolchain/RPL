//@rustc-env: RPL_PATS=tests/ui/features/session/shared_ty_hit.rpl
//@compile-flags: -Z inline-mir=false
//
// Contract: two MIR fns + `$Pair { $first: $T, $second: $U }` with only `$first` read.
// Unused `$second` / `$U` must not invent a second MIR solution.
// Two ERROR annotations per site: `$f1`/`$f2` slot permutations are distinct SessionResults.

#![allow(dead_code)]

struct Pair<T, U> {
    first: T,
    second: U,
}

fn uses_pair_a<T: Copy, U>(p: &Pair<T, U>) -> T {
    p.first
    //~^ ERROR: session shared type variables matched
    //~| ERROR: session shared type variables matched
}

fn uses_pair_b<T: Copy, U>(p: &Pair<T, U>) -> T {
    p.first
    //~^ ERROR: session shared type variables matched
    //~| ERROR: session shared type variables matched
}

fn main() {}
