#!/bin/bash

cargo cov -- show \
    --use-color \
    --ignore-filename-regex='/rustc' \
    --ignore-filename-regex='/.cargo/registry' \
    --instr-profile=stance.profdata \
    --object target/debug/deps/stance-coverage \
    --show-instantiations --show-line-counts-or-regions \
    --Xdemangler=rustfilt | less -R
