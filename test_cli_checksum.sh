#!/bin/bash

set -e
set -x

# rm -rf output

# Single test
RUST_LOG=info cargo run --release --features cuda --features checksum -- -k 20 --function zkmain --output ./output --wasm wasm/rlp.wasm setup

RUST_LOG=info cargo run --release --features cuda --features checksum -- -k 20 --function zkmain --output ./output --wasm wasm/rlp.wasm dry-run --public 133:i64 --private 0xf90424018301f8cab9010000000000000000000000000000000000000000000000008000000000020000000000100000000000000000000000000000000000000200000000000000000000000000000000000000000001002001000000000001000000000000000000000000000000020000000000040000000800000000000000000000000000000000000000000004000000000000020000000000000480000000000000000000100000000000001000000000000000000000000000000001000000000000000000000000000000000000000000000001000000004000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000f90319f901dc941f1df9f7fc939e71819f766978d8f900b816761bf842a0f6a97944f31ea060dfde0566e4167c1a1082551e64b60ecb14d599a9d023d451a00000000000000000000000000000000000000000000000000000000000007eb1b90180000000000000000000000000000000000000000000000000000fc0a072900000000000000000000000000000b4fc80aec34911c5d761259e74ae8a24c2c5d99500000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000001400000000000000000000000ad5b2a19a94f5ef600ca749c9fb37bcc0001f1cd030000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000fc0a072900000000000000000000000000000000000000000000000000000000fc0a072900000000000000000000000000000000000000000000000000000000fc0a072900000000000000000000000000000000000000000000000000000000fc2b05684202a00000000000000000000000000000000000000000000000000000000000000040102030000000000000000000000000000000000000000000000000000000000f89b941f1df9f7fc939e71819f766978d8f900b816761bf863a00109fc6f55cf40689f02fbaad7af7fe7bbac8a3d2186600afc7d3e10cac60271a00000000000000000000000000000000000000000000000000000000000007eb1a00000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000062dcd698f89b941f1df9f7fc939e71819f766978d8f900b816761bf863a00559884fd3a460db3073b7fc896cc77986f16e378210ded43186175bf646fc5fa0000000000000000000000000000000000000000000000000000fc0a072900000a00000000000000000000000000000000000000000000000000000000000007eb1a00000000000000000000000000000000000000000000000000000000062dcd698:bytes-packed

RUST_LOG=info cargo run --release --features cuda --features checksum -- -k 20 --function zkmain --output ./output --wasm wasm/rlp.wasm single-prove --public 133:i64 --private 0xf90424018301f8cab9010000000000000000000000000000000000000000000000008000000000020000000000100000000000000000000000000000000000000200000000000000000000000000000000000000000001002001000000000001000000000000000000000000000000020000000000040000000800000000000000000000000000000000000000000004000000000000020000000000000480000000000000000000100000000000001000000000000000000000000000000001000000000000000000000000000000000000000000000001000000004000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000f90319f901dc941f1df9f7fc939e71819f766978d8f900b816761bf842a0f6a97944f31ea060dfde0566e4167c1a1082551e64b60ecb14d599a9d023d451a00000000000000000000000000000000000000000000000000000000000007eb1b90180000000000000000000000000000000000000000000000000000fc0a072900000000000000000000000000000b4fc80aec34911c5d761259e74ae8a24c2c5d99500000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000001400000000000000000000000ad5b2a19a94f5ef600ca749c9fb37bcc0001f1cd030000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000fc0a072900000000000000000000000000000000000000000000000000000000fc0a072900000000000000000000000000000000000000000000000000000000fc0a072900000000000000000000000000000000000000000000000000000000fc2b05684202a00000000000000000000000000000000000000000000000000000000000000040102030000000000000000000000000000000000000000000000000000000000f89b941f1df9f7fc939e71819f766978d8f900b816761bf863a00109fc6f55cf40689f02fbaad7af7fe7bbac8a3d2186600afc7d3e10cac60271a00000000000000000000000000000000000000000000000000000000000007eb1a00000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000062dcd698f89b941f1df9f7fc939e71819f766978d8f900b816761bf863a00559884fd3a460db3073b7fc896cc77986f16e378210ded43186175bf646fc5fa0000000000000000000000000000000000000000000000000000fc0a072900000a00000000000000000000000000000000000000000000000000000000000007eb1a00000000000000000000000000000000000000000000000000000000062dcd698:bytes-packed
exit 0
RUST_LOG=info cargo run --release --features cuda --features checksum -- -k 20 --function zkmain --output ./output --wasm wasm/rlp.wasm single-verify --public 133:i64 --proof output/zkwasm.0.transcript.data

RUST_LOG=info cargo run --release --features cuda --features checksum -- -k 20 --function zkmain --output ./output --wasm wasm/rlp.wasm aggregate-prove --public 133:i64 --private 0xf90424018301f8cab9010000000000000000000000000000000000000000000000008000000000020000000000100000000000000000000000000000000000000200000000000000000000000000000000000000000001002001000000000001000000000000000000000000000000020000000000040000000800000000000000000000000000000000000000000004000000000000020000000000000480000000000000000000100000000000001000000000000000000000000000000001000000000000000000000000000000000000000000000001000000004000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000f90319f901dc941f1df9f7fc939e71819f766978d8f900b816761bf842a0f6a97944f31ea060dfde0566e4167c1a1082551e64b60ecb14d599a9d023d451a00000000000000000000000000000000000000000000000000000000000007eb1b90180000000000000000000000000000000000000000000000000000fc0a072900000000000000000000000000000b4fc80aec34911c5d761259e74ae8a24c2c5d99500000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000001400000000000000000000000ad5b2a19a94f5ef600ca749c9fb37bcc0001f1cd030000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000fc0a072900000000000000000000000000000000000000000000000000000000fc0a072900000000000000000000000000000000000000000000000000000000fc0a072900000000000000000000000000000000000000000000000000000000fc2b05684202a00000000000000000000000000000000000000000000000000000000000000040102030000000000000000000000000000000000000000000000000000000000f89b941f1df9f7fc939e71819f766978d8f900b816761bf863a00109fc6f55cf40689f02fbaad7af7fe7bbac8a3d2186600afc7d3e10cac60271a00000000000000000000000000000000000000000000000000000000000007eb1a00000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000062dcd698f89b941f1df9f7fc939e71819f766978d8f900b816761bf863a00559884fd3a460db3073b7fc896cc77986f16e378210ded43186175bf646fc5fa0000000000000000000000000000000000000000000000000000fc0a072900000a00000000000000000000000000000000000000000000000000000000000007eb1a00000000000000000000000000000000000000000000000000000000062dcd698:bytes-packed
RUST_LOG=info cargo run --release --features cuda --features checksum -- -k 20 --function zkmain --output ./output --wasm wasm/rlp.wasm aggregate-verify --proof output/aggregate-circuit.0.transcript.data  --instances output/aggregate-circuit.0.instance.data
RUST_LOG=info cargo run --release --features cuda --features checksum -- -k 20 --function zkmain --output ./output --wasm wasm/rlp.wasm solidity-aggregate-verifier --proof output/aggregate-circuit.0.transcript.data  --instances output/aggregate-circuit.0.instance.data
