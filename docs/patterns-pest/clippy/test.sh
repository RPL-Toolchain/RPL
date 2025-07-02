clear

set -eux

function uitest() {
    RPL_PATS="$1" cargo uitest -- "$2"
}

uitest "docs/patterns-pest/clippy/cast-slice-from-raw-parts.rpl"                                                   "tests/ui/clippy/cast_raw_slice_pointer_cast.rs"
uitest "docs/patterns-pest/clippy/from-raw-with-void-ptr.rpl"                                                      "tests/ui/clippy/from_raw_with_void_ptr.rs"
uitest "docs/patterns-pest/clippy/mem-replace-with-uninit.rpl:docs/patterns-pest/clippy/uninit-assumed-init.rpl"   "tests/ui/clippy/repl_uninit.rs"
uitest "docs/patterns-pest/clippy/transmuting-null.rpl"                                                            "tests/ui/clippy/transmuting_null.rs"
uitest "docs/patterns-pest/clippy/uninit-assumed-init.rpl"                                                         "tests/ui/clippy/uninit.rs"

# RPL_PATS="docs/patterns-pest" cargo uibless tests/ui/clippy
