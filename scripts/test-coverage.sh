#!/bin/bash

export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off"

# remove old building caches
rm -rf target/coverage/debug/deps/pugo_rs*

# grcov requires rust nightly at this time, 
# set TARGET_DIR to avoid nightly and stable building cache conflicts
CARGO_TARGET_DIR=./target/coverage cargo +nightly test

# generate the html report
grcov ./target/coverage/ -s . -t html --llvm --branch --ignore-not-existing -o ./target/coverage/coverage/

# open the report 
open target/coverage/coverage/index.html