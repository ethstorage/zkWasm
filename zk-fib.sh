## USAGE
## wat2wasm fib.wat
## ./zk-fib.sh fib 10 55
## ESTIMATE
## fib 10 55 -> eid 208
## fib 20 65 -> eid 398
## fib(n) = m
## formula y = 19*n + 18
## verify on fib 30 40 -> eid 588
## it indeed goes in linear

filename=$1
input_n=$2
input_m=$3
SPACE=18
echo "build wasm file ${filename}.wasm"

rm -rf ./output

export RUST_LOG=info
export RUST_BACKTRACE=1

echo "setup"
cargo run --release -- --function zkmain -k ${SPACE} --output ./output --wasm ./${filename}.wasm setup

echo "single proof"
cargo run --release -- --function zkmain -k ${SPACE} --output ./output --wasm ./${filename}.wasm single-prove --public ${input_n}:i64 --public ${input_m}:i64

echo "single verify"
cargo run --release -- --function zkmain -k ${SPACE} --output ./output --wasm ./${filename}.wasm single-verify
