clear

set -eux

export RUSTC_ICE=0
export RUST_BACKTRACE=1
RPL_ROOT="$(realpath $(realpath $0)/../..)"
export RPL_PATS="$(realpath $RPL_ROOT/docs/patterns-pest/)"

# cargo clippy

cd $RPL_ROOT
cargo lintcheck --timing --only comparison
cd -

cargo build --manifest-path $RPL_ROOT/Cargo.toml --bin rpl-driver
cargo run --manifest-path $RPL_ROOT/Cargo.toml --bin cargo-rpl $*
