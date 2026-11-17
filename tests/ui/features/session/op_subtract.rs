//@rustc-env: RPL_PATS=tests/ui/features/session/op_subtract.rpl
//@check-pass
//@compile-flags: -Z inline-mir=false
//
// Expected: no diagnostic. Both util patterns match this function with the same `$T`
// and the same labeled `'use` site, so `wide - narrow` subtracts under
// SharedEnv + DefId + NormalizedMatched (label sites). Narrow's extra unlabeled
// `$z` local is not part of NormalizedMatched, so it does not block subtraction.

#![allow(dead_code)]

fn both() -> i32 {
    let x = 1i32;
    let _z = 0i32;
    x
}

fn main() {}
