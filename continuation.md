# Run zkWASM on Continuation Mode

## What is continuation and why we need it?

In short, continuation is a feature zkVM could provide to let guest circuit to execute and generate proof with multiple small piecess rather than one giant one.

There are many reasons that why people might consider switching to continuation, including but not limited to: proving time, proving size, zkVM memory limitation and so on.

For more details, please refer this [slide](https://docs.google.com/presentation/d/1Y-B-2J6qYhyUWNDKtTwmha6_z2Z1omekL_x4CKwqSFU/edit#slide=id.g2bb7f73b7ba_0_5)

## Preparation before running

1. You have switch to a zkWASM feature with continuation, you can use this branch (frank/cont_script) or [zkWASM's own repo](https://github.com/DelphinusLab/zkWasm) to download. Run `cargo build --release --features cuda`. Note, 'release' is must. The project is vast and it takes years to build a debug version. While cuda feature is recommended for server with Nvdia GPU. Set env

```bash
export ZKWASM=/path/to/your/zkwasm
```

2. A [batcher](https://github.com/DelphinusLab/continuation-batcher) is also needed for aggregating all segments to get final proof. Build it and set env

```bash
export BATCHER=/path/to/your/batcher
```

3. Choose a WASM. If your project is on rust/js/ts/c you can choose [several sdk](https://zkwasmhub.com/) to convert it to WASM.

4. Run script

## Types of running

For continuation, there are two types of grouping:

1. (Not Recommended) You can choose equv method. This method composes all circuit segments into same one. Due to limitation of finite filed, the size of whole circuit could only be up to 2^28, which means for a larger circuit, only few circuits could be batched.

2. Use fold way. The mechanism is each time only 2 circuits participate batching process, generating a new circuit and being batched with next segment. You can choose batch_fold.sh to have a glance. But we also provide a python script:

```bash
python batch.py --help
```

to get details.

