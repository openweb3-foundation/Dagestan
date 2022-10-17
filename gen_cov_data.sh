#!/bin/bash

set -e

RUSTFLAGS="-Z instrument-coverage" \
    LLVM_PROFILE_FILE="stance-%m.profraw" \
    cargo test --tests $1 2> covtest.out

version=$(grep Running covtest.out | sed -e "s/.*stance-\(.*\))/\1/")
rm covtest.out
cp target/debug/deps/stance-"$version" target/debug/deps/stance-coverage

cargo profdata -- merge -sparse stance-*.profraw -o stance.profdata
rm stance-*.profraw

cargo cov -- report \
    --use-color \
    --ignore-filename-regex='/rustc' \
    --ignore-filename-regex='/.cargo/registry' \
    --instr-profile=stance.profdata \
    --object target/debug/deps/stance-coverage
