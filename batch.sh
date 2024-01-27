#!/bin/bash

BATCHER=/home/frank/projects/continuation-batcher/target/release/circuit-batcher
export RUST_BACKTRACE=1
export RUST_LOG=info

set -e
set -x


# circuit name for batching
CIRCUIT_NAME=fib
BATCHNAME=equvbatch

# verify generated proof for test circuits
${BATCHER} --param ./params --output ./output verify --challenge poseidon --info output/${CIRCUIT_NAME}.loadinfo.json

# batch test proofs
${BATCHER} --param ./params --output ./output batch -k 23 --challenge sha --info output/${CIRCUIT_NAME}.loadinfo.json --name ${BATCHNAME} --commits batch_strategy/equv.json

# verify generated proof for single batched circuit
${BATCHER} --param ./params --output ./output verify --challenge sha --info output/${BATCHNAME}.loadinfo.json