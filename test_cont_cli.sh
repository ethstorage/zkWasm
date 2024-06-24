CLI=/home/po/now/zkWasm_cont/target/release/zkwasm-cli
export CUDA_VISIBLE_DEVICES=0
test_continuation_cli() {
    cargo build --release --features continuation,perf,cuda 
    rm -rf params/*.data params/*.config output
    $CLI --params ./params fibonacci setup -k 22
    $CLI --params ./params fibonacci dry-run --wasm crates/zkwasm/wasm/fibonacci.wasm --public 25:i64 --output ./output
    $CLI --params ./params fibonacci prove --wasm crates/zkwasm/wasm/fibonacci.wasm --public 25:i64 --output ./output
    $CLI --params ./params fibonacci verify --output ./output
}

test_continuation_cli