clear

set -eux

RPL_PATS="docs/patterns-pest/CVE-2018-21000.rpl" cargo uitest -- "tests/ui/cve_2018_21000"
RPL_PATS="docs/patterns-pest/cve-2019-15548.rpl" cargo uitest -- "tests/ui/cve_2019_15548"
RPL_PATS="docs/patterns-pest/cve-2020-25016.rpl" cargo uitest -- "tests/ui/cve_2020_25016"
RPL_PATS="docs/patterns-pest/cve-2021-27376.rpl" cargo uitest -- "tests/ui/cve_2021_27376"
RPL_PATS="docs/patterns-pest/cast-size-different-sizes.rpl" cargo uitest -- "tests/ui/cast_size_different_sizes"
RPL_PATS="docs/patterns-pest/unsound-collection-transmute.rpl" cargo uitest -- "tests/ui/unsound_collection_transmute"
