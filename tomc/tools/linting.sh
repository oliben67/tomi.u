#!/bin/bash

cargo +nightly fmt
cargo +nightly clippy --fix --allow-dirty --allow-staged -- -D warnings
cargo +nightly clippy -- -D warnings
# cargo +nightly test --all-features -- --test-threads=1  