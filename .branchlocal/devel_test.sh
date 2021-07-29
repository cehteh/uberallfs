#!/bin/sh
export CARGO_INCREMENTAL=0
cargo errors && cargo test -- --nocapture
