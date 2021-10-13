#!/bin/sh
# branch:master
# branch:devel
# branch:feature/*
export CARGO_INCREMENTAL=0
cargo fmt --all -- --check && cargo errors && cargo test -- --nocapture
echo $?
