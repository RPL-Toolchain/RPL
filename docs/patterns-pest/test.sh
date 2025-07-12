clear

set -eux

function uitest() {
    RPL_PATS="$1" cargo uitest -- "$2"
}

uitest "docs/patterns-pest/cast-size-different-sizes.rpl"                                                         "tests/ui/cast_size_different_sizes"
uitest "docs/patterns-pest/CVE-2018-20992.rpl"                                                                    "tests/ui/cve_2018_20992" # Duplicates
uitest "docs/patterns-pest/CVE-2018-21000.rpl"                                                                    "tests/ui/cve_2018_21000"
uitest "docs/patterns-pest/CVE-2019-15548.rpl"                                                                    "tests/ui/cve_2019_15548" # Order changed
uitest "docs/patterns-pest/CVE-2019-16138.rpl:docs/patterns-pest/private-or-generic-function-marked-inline.rpl"   "tests/ui/cve_2019_16138" # Order changed
uitest "docs/patterns-pest/CVE-2020-25016.rpl:docs/patterns-pest/private-or-generic-function-marked-inline.rpl"   "tests/ui/cve_2020_25016" # Order changed
uitest "docs/patterns-pest/CVE-2020-25795.rpl"                                                                    "tests/ui/cve_2020_25795"
uitest "docs/patterns-pest/CVE-2020-35860.rpl:docs/patterns-pest/private-or-generic-function-marked-inline.rpl"   "tests/ui/cve_2020_35860"
uitest "docs/patterns-pest/CVE-2020-35862.rpl:docs/patterns-pest/private-or-generic-function-marked-inline.rpl"   "tests/ui/cve_2020_35862" # Order changed
uitest "docs/patterns-pest/CVE-2020-35873.rpl"                                                                    "tests/ui/cve_2020_35873"
uitest "docs/patterns-pest/CVE-2020-35877.rpl"                                                                    "tests/ui/cve_2020_35877"
uitest "docs/patterns-pest/CVE-2020-35881.rpl"                                                                    "tests/ui/cve_2020_35881"
uitest "docs/patterns-pest/CVE-2020-35881.rpl"                                                                    "tests/ui/cve_2020_35881_dyn_derive"
uitest "docs/patterns-pest/allow-unchecked.rpl"                                                                   "tests/ui/cve_2020_35887"
uitest "docs/patterns-pest/CVE-2020-35888.rpl"                                                                    "tests/ui/cve_2020_35888" # Duplicates
uitest "docs/patterns-pest/CVE-2020-35888.rpl"                                                                    "tests/ui/cve_2020_35888_simplified"
uitest "docs/patterns-pest/CVE-2020-35892-3.rpl:docs/patterns-pest/private-or-generic-function-marked-inline.rpl" "tests/ui/cve_2020_35892_3" # Too complex diagnostic messages, need a new mechanism for it
uitest "docs/patterns-pest/CVE-2020-35898-9.rpl"                                                                  "tests/ui/cve_2020_35898_9" # Unreasonable diagnostic messages, fixable but I'm not sure if it's worth it
uitest "docs/patterns-pest/CVE-2020-35901-2.rpl"                                                                  "tests/ui/cve_2020_35901_2"
uitest "docs/patterns-pest/CVE-2020-35907.rpl"                                                                    "tests/ui/cve_2020_35907"
uitest "docs/patterns-pest/CVE-2021-25904.rpl"                                                                    "tests/ui/cve_2021_25904"
uitest "docs/patterns-pest/CVE-2021-25905.rpl:docs/patterns-pest/CVE-2021-29941-2.rpl"                            "tests/ui/cve_2021_25905"
uitest "docs/patterns-pest/CVE-2021-27376.rpl"                                                                    "tests/ui/cve_2021_27376" # Order changed
uitest "docs/patterns-pest/CVE-2021-29941-2.rpl:docs/patterns-pest/CVE-2019-16138.rpl"                            "tests/ui/cve_2021_29941_2" # Order changed
uitest "docs/patterns-pest/CVE-2019-16138.rpl"                                                                    "tests/ui/cve_2021_45688"
uitest "docs/patterns-pest/CVE-2022-23639.rpl"                                                                    "tests/ui/cve_2022_23639"
uitest "docs/patterns-pest/CVE-2024-27284.rpl"                                                                    "tests/ui/cve_2024_27284"
uitest "docs/patterns-pest/allow-unchecked.rpl"                                                                   "tests/ui/cve_2025_48755" # Unreasonable diagnostic messages, fixable but I'm not sure if it's worth it
uitest "docs/patterns-pest/private-or-generic-function-marked-inline.rpl"                                         "tests/ui/generic_functions_marked_inline"
uitest "docs/patterns-pest/private-or-generic-function-marked-inline.rpl"                                         "tests/ui/private_function_marked_inline" # Order changed
uitest "docs/patterns-pest/allow-unchecked.rpl"                                                                   "tests/ui/std/alloc"
uitest "docs/patterns-pest/manually-drop.rpl"                                                                     "tests/ui/std/mem/ManuallyDrop"
uitest "docs/patterns-pest/transmute-int-to-ptr.rpl:docs/patterns-pest/transmute-to-bool.rpl"                     "tests/ui/std/mem/transmute"
uitest "docs/patterns-pest/clippy/unsound-collection-transmute.rpl"                                               "tests/ui/unsound_collection_transmute"

RPL_PATS="docs/patterns-pest" cargo uibless
