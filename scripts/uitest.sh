clear

set -eux

export RUSTC_ICE=0
export RUST_BACKTRACE=1
export RUSTC_LOG_COLOR=always
# export RUSTC_LOG="rpl=info"
export RPL_PATS="$1"

cargo uitest -- "$2"
