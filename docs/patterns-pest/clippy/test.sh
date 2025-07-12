clear

set -eux

function uitest() {
    RPL_PATS="$1" cargo uitest -- "$2"
}

uitest "docs/patterns-pest/clippy/cast-slice-different-sizes.rpl"                                                  "tests/ui/clippy/cast_slice_different_sizes.rs"
uitest "docs/patterns-pest/clippy/cast-slice-from-raw-parts.rpl"                                                   "tests/ui/clippy/cast_raw_slice_pointer_cast.rs"
uitest "docs/patterns-pest/clippy/from-raw-with-void-ptr.rpl"                                                      "tests/ui/clippy/from_raw_with_void_ptr.rs"
uitest "docs/patterns-pest/clippy/not-unsafe-ptr-arg-deref.rpl"                                                    "tests/ui/clippy/functions.rs"
uitest "docs/patterns-pest/clippy/mem-replace-with-uninit.rpl:docs/patterns-pest/clippy/uninit-assumed-init.rpl"   "tests/ui/clippy/repl_uninit.rs"
uitest "docs/patterns-pest/clippy/transmute-int-to-non-zero.rpl"                                                   "tests/ui/clippy/transmute_int_to_non_zero.rs"
uitest "docs/patterns-pest/clippy/transmute-null-to-fn.rpl"                                                        "tests/ui/clippy/transmute_null_to_fn.rs"
uitest "docs/patterns-pest/clippy/transmuting-null.rpl"                                                            "tests/ui/clippy/transmuting_null.rs"
uitest "docs/patterns-pest/clippy/uninit-assumed-init.rpl"                                                         "tests/ui/clippy/uninit.rs"
uitest "docs/patterns-pest/clippy/unsound-collection-transmute.rpl"                                                "tests/ui/clippy/transmute_collection.rs"
uitest "docs/patterns-pest/clippy/wrong-transmute.rpl"                                                             "tests/ui/clippy/transmute_32bit.rs"
uitest "docs/patterns-pest/clippy/wrong-transmute.rpl"                                                             "tests/ui/clippy/transmute_64bit.rs"
# RPL_PATS="docs/patterns-pest" cargo uibless tests/ui/clippy
