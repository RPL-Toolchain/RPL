//! Path tables and helpers mirroring Rudra's Unsafe Dataflow bypass / sink lists.
//! See <https://github.com/sslab-gatech/Rudra/blob/master/src/paths.rs>.

use rustc_hir::def_id::DefId;
use rustc_middle::ty::TyCtxt;

/// Strong lifetime-bypass path suffixes (crate-independent).
const STRONG_BYPASS_SUFFIXES: &[&[&str]] = &[
    &["ptr", "read"],
    &["ptr", "const_ptr", "read"],
    &["ptr", "copy"],
    &["ptr", "copy_nonoverlapping"],
    &["intrinsics", "copy"],
    &["intrinsics", "copy_nonoverlapping"],
    // `-Z inline-mir=true` inlines `intrinsics::copy` (re-exported as `ptr::copy`) down to the
    // nested `rust-intrinsic` function, whose def-path is `core::intrinsics::copy::copy`
    // (extra trailing segment). (`copy_nonoverlapping` has no such entry because inlining
    // lowers it to a `NonDivergingIntrinsic::CopyNonOverlapping` *statement*, not a call.)
    &["intrinsics", "copy", "copy"],
    &["vec", "Vec", "set_len"],
    &["vec", "Vec", "from_raw_parts"],
];

/// Weak lifetime-bypass path suffixes.
const WEAK_BYPASS_SUFFIXES: &[&[&str]] = &[
    &["intrinsics", "transmute"],
    &["ptr", "write"],
    &["ptr", "mut_ptr", "write"],
    &["ptr", "const_ptr", "as_ref"],
    &["ptr", "mut_ptr", "as_mut"],
    &["ptr", "non_null", "NonNull", "as_ref"],
    &["ptr", "non_null", "NonNull", "as_mut"],
    &["slice", "get_unchecked"],
    &["slice", "get_unchecked_mut"],
    &["ptr", "slice_from_raw_parts"],
    &["ptr", "slice_from_raw_parts_mut"],
    &["slice", "from_raw_parts"],
    &["slice", "from_raw_parts_mut"],
];

const GENERIC_DROP_SUFFIXES: &[&[&str]] = &[&["ptr", "drop_in_place"], &["ptr", "mut_ptr", "drop_in_place"]];

/// `set_len` predicate: match any method whose name is `set_len` (e.g. `Vec::set_len`,
/// `SmallVec::set_len`, or a container's own `set_len`), not just `std::vec::Vec::set_len`.
const SET_LEN_SUFFIX: &[&str] = &["set_len"];
const PTR_READ_SUFFIXES: &[&[&str]] = &[&["ptr", "read"], &["ptr", "const_ptr", "read"]];
const PTR_WRITE_SUFFIXES: &[&[&str]] = &[&["ptr", "write"], &["ptr", "mut_ptr", "write"]];

/// Named components of `def_id`'s path, including the crate name.
pub fn def_path_names(tcx: TyCtxt<'_>, def_id: DefId) -> Vec<String> {
    let mut names = vec![tcx.crate_name(def_id.krate).to_string()];
    for data in &tcx.def_path(def_id).data {
        if let Some(name) = data.data.get_opt_name() {
            names.push(name.as_str().to_string());
        }
    }
    names
}

fn path_ends_with(path: &[String], suffix: &[&str]) -> bool {
    path.len() >= suffix.len()
        && path[path.len() - suffix.len()..]
            .iter()
            .zip(suffix)
            .all(|(a, b)| a == b)
}

fn path_matches_any(path: &[String], suffixes: &[&[&str]]) -> bool {
    suffixes.iter().any(|suffix| path_ends_with(path, suffix))
}

pub fn is_strong_bypass(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    path_matches_any(&def_path_names(tcx, def_id), STRONG_BYPASS_SUFFIXES)
}

pub fn is_weak_bypass(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    path_matches_any(&def_path_names(tcx, def_id), WEAK_BYPASS_SUFFIXES)
}

pub fn is_lifetime_bypass(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    is_strong_bypass(tcx, def_id) || is_weak_bypass(tcx, def_id)
}

pub fn is_generic_drop(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    path_matches_any(&def_path_names(tcx, def_id), GENERIC_DROP_SUFFIXES)
}

pub fn is_set_len(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    path_ends_with(&def_path_names(tcx, def_id), SET_LEN_SUFFIX)
}

pub fn is_ptr_read(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    path_matches_any(&def_path_names(tcx, def_id), PTR_READ_SUFFIXES)
}

pub fn is_ptr_write(tcx: TyCtxt<'_>, def_id: DefId) -> bool {
    path_matches_any(&def_path_names(tcx, def_id), PTR_WRITE_SUFFIXES)
}
