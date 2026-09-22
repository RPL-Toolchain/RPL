//! Unit tests for the cartesian-product helper.
//!
//! These tests verify the four corner-cases specified in Task 11:
//! - empty input yields one empty combo
//! - single factor yields one combo per element
//! - two factors yield the full cross-product
//! - one empty factor folds the product to zero combos (the fold-on-no-instances mechanism)
#![feature(rustc_private)]

use rpl_driver::cartesian;

#[test]
fn empty_input_yields_one_empty_combo() {
    let combos: Vec<Vec<usize>> = cartesian(std::iter::empty::<std::vec::IntoIter<usize>>()).collect();
    assert_eq!(combos, vec![Vec::<usize>::new()]);
}

#[test]
fn single_factor_yields_one_combo_per_element() {
    let factors = vec![vec![1usize, 2, 3]];
    let iters: Vec<_> = factors.into_iter().map(|v| v.into_iter()).collect();
    let combos: Vec<Vec<usize>> = cartesian(iters).collect();
    assert_eq!(combos.len(), 3);
}

#[test]
fn two_factors_two_three_yields_six() {
    let factors = vec![vec![1usize, 2], vec![10, 20, 30]];
    let iters: Vec<_> = factors.into_iter().map(|v| v.into_iter()).collect();
    let combos: Vec<Vec<usize>> = cartesian(iters).collect();
    assert_eq!(combos.len(), 6);
}

#[test]
fn one_empty_factor_yields_zero_combos() {
    let factors: Vec<Vec<usize>> = vec![vec![1, 2], vec![]];
    let iters: Vec<_> = factors.into_iter().map(|v| v.into_iter()).collect();
    let combos: Vec<Vec<usize>> = cartesian(iters).collect();
    assert!(combos.is_empty(), "an empty factor folds the product to empty");
}
