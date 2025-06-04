clear

set -eux

# docs/patterns-pest/CVE-2018-21000.rpl
# docs/patterns-pest/CVE-2019-15548.rpl
# docs/patterns-pest/CVE-2020-25016.rpl
# docs/patterns-pest/CVE-2021-27376.rpl
# docs/patterns-pest/cast-size-different-sizes.rpl
# docs/patterns-pest/unsound-collection-transmute.rpl

RPL_PATS="docs/patterns-pest/" cargo uitest -- \
    "tests/ui/cve_2018_21000" \
    "tests/ui/cve_2019_15548" \
    "tests/ui/cve_2020_25016" \
    "tests/ui/cve_2021_27376" \
    "tests/ui/cast_size_different_sizes" \
    "tests/ui/unsound_collection_transmute"
