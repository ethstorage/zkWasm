#!/bin/bash

CLI=./target/release/delphinus-cli

set -e
set -x

test_default_cli() {
    cargo build --release
    rm -rf params output
    $CLI --params ./params wasm_output setup --wasm ./crates/zkwasm/wasm/wasm_output.wasm
}

test_uniform_circuit_cli() {
    cargo build --release --features uniform-circuit
    rm -rf params output
    $CLI --params ./params wasm_output setup
}

test_continuation_cli() {
    cargo build --release --features continuation
    rm -rf params output
    $CLI --params ./params wasm_output setup
}

test_default_cli
test_uniform_circuit_cli
test_continuation_cli


exit 0

$CLI --params ./params wasm_output dry-run --wasm crates/zkwasm/wasm/wasm_output.wasm --public 133:i64 --public 2:i64 --output ./output
$CLI --params ./params wasm_output prove --wasm crates/zkwasm/wasm/wasm_output.wasm --public 133:i64 --public 2:i64 --output ./output --mock
$CLI --params ./params wasm_output verify --output ./output
