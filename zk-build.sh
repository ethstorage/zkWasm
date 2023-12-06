filename=$1
SPACE=18
echo "build wasm file ${filename}.wasm"

rm -rf ./output

export RUST_LOG=info
export RUST_BACKTRACE=1

echo "setup"
cargo run --release -- --function zkmain -k ${SPACE} --output ./output --wasm ./${filename}.wasm setup

echo "single proof"
cargo run --release -- --function zkmain -k ${SPACE} --output ./output --wasm ./${filename}.wasm single-prove

echo "single verify"
cargo run --release -- --function zkmain -k ${SPACE} --output ./output --wasm ./${filename}.wasm single-verify
