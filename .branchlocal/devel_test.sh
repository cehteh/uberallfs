#!/bin/sh
cargo errors && cargo test -- --nocapture
