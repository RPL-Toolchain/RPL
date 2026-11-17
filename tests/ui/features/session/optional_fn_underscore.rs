//@rustc-env: RPL_PATS=tests/ui/features/session/optional_fn_underscore.rpl
//@compile-flags: -Z inline-mir=false
//
// Expected: only `uses_pair` lints; optional `fn _` may skip unrelated items.
// Only `$first` is read; unused `$second` must not fork a second solution.

#![allow(dead_code)]

struct Pair<T, U> {
    first: T,
    second: U,
}

fn uses_pair<T: Copy, U>(p: &Pair<T, U>) -> T {
    p.first
    //~^ ERROR: session optional fn _ matched required slot
}

fn unrelated_a() {}

fn unrelated_b(_: i32) {}

fn main() {}
