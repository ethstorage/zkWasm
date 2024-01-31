#!/bin/bash

BATCHER=/home/frank/projects/continuation-batcher/target/release/circuit-batcher
export RUST_BACKTRACE=1
export RUST_LOG=info

set -e
set -x


# circuit name for batching
CIRCUIT_NAME=fib
BATCHNAME=foldbatch
COUNT=0

# verify generated proof for test circuits
${BATCHER} --param ./params --output ./output verify --challenge poseidon --info output/${CIRCUIT_NAME}.loadinfo.json

# batch test proofs
${BATCHER} --param ./params --output ./output batch -k 23 --challenge poseidon --info output/${CIRCUIT_NAME}.loadinfo.json --name ${BATCHNAME}_${COUNT} --commits batch_strategy/fold/fold${COUNT}.json

# verify generated proof for single batched circuit
${BATCHER} --param ./params --output ./output verify --challenge poseidon --info output/${BATCHNAME}_${COUNT}.loadinfo.json

# count bumps up
PREV_COUNT=${COUNT}
COUNT=$((COUNT+1))

# batch second
${BATCHER} --param ./params --output ./output batch -k 23 --challenge poseidon --info output/${CIRCUIT_NAME}.loadinfo.json output/${BATCHNAME}_${PREV_COUNT}.loadinfo.json --name ${BATCHNAME}_${COUNT} --commits batch_strategy/fold/fold${COUNT}.json

# verify generated proof for single batched circuit
${BATCHER} --param ./params --output ./output verify --challenge poseidon --info output/${BATCHNAME}_${COUNT}.loadinfo.json