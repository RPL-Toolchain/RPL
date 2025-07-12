clear

set -eux

export RUSTC_ICE=0
export RUST_BACKTRACE=1
export RUSTC_LOG_COLOR=always
export RUSTC_LOG="${3:-rpl=info}"
export RPL_PATS="$1"

cargo +rpl-dbg run --bin rpl-driver -- -L target/debug/deps -Z no-codegen "$2" -W unconditional_panic -A unused ${@:4} 2>&1 | tee .ansi
