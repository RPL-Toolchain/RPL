//! Built-in RPL patterns embedded from `docs/patterns-pest`.
//!
//! Maintain this list by hand when adding/removing `.rpl` files.
//! Order must match the filesystem: categories `clippy`, `codeql`, `cve`, `ub`
//! (in that order), and lexicographic file names within each category.
//! CI enforces this via `scripts/check_rpl_patterns.py`.

/// This macro will return a tuple of the path and the content of the file.
///
/// Please pass a path related to `docs/patterns-pest`.
macro_rules! default_pattern {
    ($path:literal) => {
        (
            concat!("docs/patterns-pest/", $path),
            include_str!(concat!("../../../docs/patterns-pest/", $path)),
        )
    };
}

macro_rules! default_patterns {
    ($($name:literal),* $(,)?) => {
        vec![$(
            default_pattern!($name),
        )*]
    };
}

pub fn patterns() -> Vec<(&'static str, &'static str)> {
    default_patterns![
        // Clippy lints
        "clippy/cast-slice-different-sizes.rpl",
        "clippy/cast-slice-from-raw-parts.rpl",
        "clippy/eager-transmute.rpl",
        "clippy/from-raw-with-void-ptr.rpl",
        "clippy/mem-replace-with-uninit.rpl",
        "clippy/mut-from-ref.rpl",
        "clippy/not-unsafe-ptr-arg-deref.rpl",
        "clippy/ptr-offset-with-cast.rpl",
        "clippy/size-of-in-element-count.rpl",
        "clippy/swap-ptr-to-ref.rpl",
        "clippy/transmute-int-to-non-zero.rpl",
        "clippy/transmute-null-to-fn.rpl",
        "clippy/transmuting-null.rpl",
        "clippy/uninit-assumed-init.rpl",
        "clippy/uninit-vec.rpl",
        "clippy/unsound-collection-transmute.rpl",
        "clippy/wrong-transmute.rpl",
        "clippy/zst-offset.rpl",
        // CodeQL patterns
        "codeql/access-after-lifetime-ended.rpl",
        "codeql/access-invalid-pointer.rpl",
        "codeql/ctor-initialization.rpl",
        // CVE patterns
        "cve/CVE-2018-20992.rpl",
        "cve/CVE-2018-21000.rpl",
        "cve/CVE-2019-15543.rpl",
        "cve/CVE-2019-15548.rpl",
        "cve/CVE-2019-15551.rpl",
        "cve/CVE-2019-16138.rpl",
        "cve/CVE-2020-25016.rpl",
        "cve/CVE-2020-35860.rpl",
        "cve/CVE-2020-35862.rpl",
        "cve/CVE-2020-35873.rpl",
        "cve/CVE-2020-35877.rpl",
        "cve/CVE-2020-35881.rpl",
        "cve/CVE-2020-35887.rpl",
        "cve/CVE-2020-35888.rpl",
        "cve/CVE-2020-35892-3.rpl",
        "cve/CVE-2020-35898-9.rpl",
        "cve/CVE-2020-35901-2.rpl",
        "cve/CVE-2020-35907.rpl",
        "cve/CVE-2020-35916.rpl",
        "cve/CVE-2020-35923.rpl",
        "cve/CVE-2021-25904.rpl",
        "cve/CVE-2021-25905.rpl",
        "cve/CVE-2021-26307.rpl",
        "cve/CVE-2021-27376.rpl",
        "cve/CVE-2021-29941-2.rpl",
        "cve/CVE-2022-23639.rpl",
        "cve/CVE-2024-27284.rpl",
        // Rudra patterns
        "rudra/panic-safety.rpl",
        // Common patterns based on Rust's UB
        "ub/allow-unchecked.rpl",
        "ub/manually-drop.rpl",
        "ub/private-or-generic-function-marked-inline.rpl",
        "ub/transmute-int-to-ptr.rpl",
        "ub/transmute-to-bool.rpl",
    ]
}
