//@rustc-env: RPL_PATS=tests/ui/features/session/sig_not_arity.rpl
//@compile-flags: -Z inline-mir=false
//
// Expected: only the `$Pair` functions lint; `distractor(_: i32)` must not.
// Two ERROR annotations per site: `$f1`/`$f2` slot permutations are distinct SessionResults.

#![allow(dead_code)]

struct Pair<T, U> {
    first: T,
    second: U,
}

fn uses_pair_a(_: Pair<u8, u16>) {
    //~^ ERROR: session signature slot matched on $Pair
    //~| ERROR: session signature slot matched on $Pair
}

fn uses_pair_b(_: Pair<u8, u16>) {
    //~^ ERROR: session signature slot matched on $Pair
    //~| ERROR: session signature slot matched on $Pair
}

/// Unary distractor: arity matches the pattern but the type does not.
fn distractor(_: i32) {}

fn main() {}
