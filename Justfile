#cli := env_var_or_default('ZKWASM_CLI', 'zkwasm-cli-x86')

gk-dr:
  cargo run -- --function zkmain --output ./output --param ./param --host host --wasm ../go-keccak256/keccak256.wasm dry-run

ghk-dr:
  cargo run -- --function zkmain --output ./output --param ./param --host host --wasm ../go-host-keccak256/keccak256.wasm dry-run

rk-dr:
  cargo run -- --function zkmain --output ./output --param ./param --host host --wasm ../rust-keccak256/pkg/rust-sdk-test.wasm dry-run

rhk-dr:
  cargo run -- --function zkmain --output ./output --param ./param --host host --wasm ../rust-host-keccak256/pkg/rust-sdk-test.wasm dry-run

opb-dr:
  cargo run -- --function zkmain --output ./output --param ./param --host host --wasm ../optimism/op-program/bin/op-program-client-test.wasm dry-run --private ~/last_op/EthStorage-Grant/preimages-test.bin:file

opl-dr:
  cargo run -- --function zkmain --output ./output --param ./param --host host --wasm ../optimism/op-program/bin/op-program-client-test.wasm dry-run --private ~/last_op/EthStorage-Grant/preimages-test-little.bin:file

oplf-dr:
  cargo run -- --function zkmain --output ./output --param ./param --host host --wasm ../optimism/op-program/bin/op-program-client.wasm dry-run --private ~/last_op/EthStorage-Grant/preimages-little.bin:file


go-dr:
  just gk-dr
  just ghk-dr

rust-dr:
  just rk-dr
  just rhk-dr
