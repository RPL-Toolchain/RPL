#!/usr/bin/env zsh

clear

set -eux

export RUSTC_ICE=0
export RUST_BACKTRACE=1
RPL_ROOT="$(realpath $(dirname $0)/..)"

cd $RPL_ROOT

export RPL_PATS="$(realpath $RPL_ROOT/docs/patterns-pest/clippy)"

for i in {1..5}
do
    cargo lintcheck --timing -j 24 --crates-toml crates/lintcheck/1000.toml
    cp ./lintcheck-logs/1000_logs.txt ./lintcheck-logs/1000_logs.clippy.$i.txt
done

export RPL_PATS="$(realpath $RPL_ROOT/docs/patterns-pest)"

for i in {1..5}
do
    cargo lintcheck --timing -j 24 --crates-toml crates/lintcheck/1000.toml
    cp ./lintcheck-logs/1000_logs.txt ./lintcheck-logs/1000_logs.all.$i.txt
done
