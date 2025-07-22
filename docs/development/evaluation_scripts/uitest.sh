clear

set -eux

export RUSTC_ICE=0
export RUST_BACKTRACE=1
export RUSTC_LOG_COLOR=always
# export RUSTC_LOG="rpl=trace"
export RPL_PATS="$1"

case ${4:-} in
    '')
        ;;
    *)
        export TOOLCHAIN="+rpl-dbg" # Check toolchain name on your system
        export RUSTC_LOG="$4"       # Such as `rpl=info`
        ;;
esac


case ${3:-'--test'} in
    --test)
        cargo ${TOOLCHAIN:-} uitest -- "${2:-}" | tee .ansi
        ;;
    --bless)
        cargo ${TOOLCHAIN:-} uibless -- "${2:-}" | tee .ansi
        ;;
    *)
        echo "Invalid mode."
        exit -1
        ;;
esac
