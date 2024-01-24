#!/bin/bash

CLI=./target/release/delphinus-cli

set -e
set -x

CUDA="--features cuda"

test_default_cli() {
    cargo build --release $CUDA
    rm -rf params output
    $CLI --params ./params wasm_output setup --wasm ./crates/zkwasm/wasm/wasm_output.wasm
    $CLI --params ./params wasm_output dry-run --wasm crates/zkwasm/wasm/wasm_output.wasm --public 133:i64 --public 2:i64 --output ./output
    $CLI --params ./params wasm_output prove --wasm crates/zkwasm/wasm/wasm_output.wasm --public 133:i64 --public 2:i64 --output ./output
    $CLI --params ./params wasm_output verify --output ./output
}

test_uniform_circuit_cli() {
    cargo build --release --features uniform-circuit $CUDA
    rm -rf params output
    $CLI --params ./params wasm_output setup
    $CLI --params ./params wasm_output dry-run --wasm crates/zkwasm/wasm/wasm_output.wasm --public 133:i64 --public 2:i64 --output ./output
    $CLI --params ./params wasm_output prove --wasm crates/zkwasm/wasm/wasm_output.wasm --public 133:i64 --public 2:i64 --output ./output
    $CLI --params ./params wasm_output verify --output ./output
}

test_continuation_cli() {
    cargo build --release --features continuation $CUDA
    rm -rf params output
    $CLI --params ./params wasm_output setup
    $CLI --params ./params wasm_output dry-run --wasm crates/zkwasm/wasm/fibonacci.wasm --public 25:i64 --output ./output
    $CLI --params ./params wasm_output prove --wasm crates/zkwasm/wasm/fibonacci.wasm --public 25:i64 --output ./output -m
    $CLI --params ./params wasm_output verify --output ./output
}

test_default_cli
test_uniform_circuit_cli
test_continuation_cli

