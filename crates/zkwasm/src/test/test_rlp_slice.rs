use std::sync::Arc;
use std::sync::Mutex;

use crate::continuation::slice::Slice;
use crate::loader::ExecutionArg;
use crate::loader::ZkWasmLoader;
use crate::runtime::ExecutionResult;
use specs::Tables;

use anyhow::Result;
use halo2_proofs::pairing::bn256::Bn256;
use halo2_proofs::pairing::bn256::Fr;
use wasmi::RuntimeValue;

fn generate_wasm_result(
    dump_table: bool,
) -> Result<(ZkWasmLoader<Bn256>, Vec<Fr>, ExecutionResult<RuntimeValue>)> {
    let public_inputs = vec![133];
    let private_inputs: Vec<u64> = vec![
        14625441452057167097,
        441,
        0,
        0,
        144115188084244480,
        17592186044416,
        0,
        0,
        2,
        0,
        281474976710656,
        72057594037928224,
        0,
        144115188075855872,
        4398046511104,
        2048,
        0,
        288230376151711744,
        562949953421312,
        36033195065475072,
        0,
        1152921504606846992,
        0,
        72057594037927936,
        0,
        0,
        72057594037927936,
        274877906944,
        0,
        8192,
        0,
        0,
        0,
        142172368092004352,
        10663670667014018268,
        15598333267600830878,
        4825637194728734969,
        11537926770794296992,
        8941585237026987872,
        1060144843738714138,
        15286290987074524363,
        41041,
        0,
        0,
        0,
        549784760702,
        0,
        0,
        13839280179932823552,
        9466528,
        0,
        1245741926200423424,
        9993052845762533317,
        643603743268,
        0,
        0,
        0,
        687194767360,
        0,
        0,
        0,
        274894684160,
        0,
        17752714368831347629,
        14734568103978781184,
        16340025600,
        0,
        0,
        0,
        17179869184,
        0,
        0,
        13839280179932823552,
        9466528,
        0,
        0,
        13839280179932823552,
        9466528,
        0,
        0,
        13839280179932823552,
        9466528,
        0,
        0,
        13983395368008679424,
        180934170288,
        0,
        0,
        0,
        216736848758702080,
        0,
        0,
        0,
        10708425217887174656,
        8187143887307480351,
        70325280878010241,
        117203507575396024,
        11486502108844260361,
        13539931197926996738,
        18161434576524511916,
        11561024771253616253,
        0,
        0,
        0,
        12789659991778787328,
        160,
        0,
        0,
        0,
        40960,
        0,
        0,
        15880255236061790208,
        17950538412901046486,
        8547692942764276983,
        8509190860294355049,
        5730928406529570843,
        18210150271972058323,
        3994395479395232905,
        6563862530498629762,
        688805136118,
        0,
        0,
        13839280179932823552,
        175921869910688,
        0,
        0,
        0,
        45231150997700608,
        0,
        0,
        0,
        43020438485336064,
    ];

    let wasm = std::fs::read("wasm/rlp.wasm").unwrap();

    let loader = ZkWasmLoader::<Bn256>::new(18, wasm, vec![])?;

    let execution_result = loader.run(ExecutionArg {
        public_inputs,
        private_inputs,
        context_inputs: vec![],
        context_outputs: Arc::new(Mutex::new(vec![])),
        output_dir: Some(std::env::current_dir().unwrap()),
        dump_table,
    })?;

    let instances = execution_result
        .public_inputs_and_outputs
        .iter()
        .map(|v| (*v).into())
        .collect();
    Ok((loader, instances, execution_result))
}

fn test_slices() -> Result<()> {
    let (loader, instances, execution_result) = generate_wasm_result(false)?;
    let mut slices = loader.slice(execution_result).into_iter();

    let mut index = 0;

    while let Some(slice) = slices.next() {
        println!("slice {}", index);

        if index != 0 {
            let circuit = slice.build_circuit();

            loader.mock_test(&circuit, &instances)?;
            loader.bench_test(circuit, &instances);
        }

        index += 1;
    }

    Ok(())
}

fn test_rpl_slice_from_file() -> Result<()> {
    let (loader, instances, execution_result) = generate_wasm_result(false)?;
    let mut slices = loader.slice(execution_result).into_iter();

    let mut index = 0;
    while let Some(slice) = slices.next() {
        let mut dir = std::env::current_dir().unwrap();
        // push a namespace to avoid conflict with test_rpl_slice_dump when testing concurrently
        dir.push("full_run_dump");
        dir.push(index.to_string());
        slice.write_flexbuffers(Some(dir));
        index += 1;
    }

    let last_slice_index = index - 1;
    while index > 0 {
        index -= 1;
        let mut dir = std::env::current_dir().unwrap();
        dir.push("full_run_dump");
        dir.push(index.to_string());

        let table = Tables::load(
            dir.clone(),
            index == last_slice_index,
            specs::FileType::FLEXBUFFERS,
        );
        let slice = Slice::new(table, slices.capability());
        let circuit = slice.build_circuit();
        loader.mock_test(&circuit, &instances)?;

        std::fs::remove_dir_all(dir).unwrap();
    }

    Ok(())
}

fn test_rpl_slice_dump() -> Result<()> {
    // dump slice while running
    let (_, _instances, _) = generate_wasm_result(true)?;
    // dump slice after a run
    let (loader, _, execution_result) = generate_wasm_result(false)?;
    let mut slices = loader.slice(execution_result).into_iter();

    let last_slice_index = slices.num_slices() - 1;
    let mut index = 0;
    while let Some(slice) = slices.next() {
        // load slice from running dump
        let mut dir = std::env::current_dir().unwrap();
        dir.push(index.to_string());
        let table = Tables::load(
            dir.clone(),
            index == last_slice_index,
            specs::FileType::FLEXBUFFERS,
        );
        let loaded_slice = Slice::new(table, slices.capability());

        // make sure slices generated from memory and during running is the same
        assert_eq!(slice, loaded_slice);

        let circuit = loaded_slice.build_circuit();
        loader.mock_test(&circuit, &_instances)?;

        std::fs::remove_dir_all(dir).unwrap();
        index += 1;
    }

    Ok(())
}

mod tests {
    use super::*;

    #[test]
    fn test_rlp_slice_mock() {
        test_slices().unwrap();
    }

    #[test]
    fn test_rpl_slice_from_file_mock() {
        test_rpl_slice_from_file().unwrap();
    }

    #[test]
    fn test_rpl_slice_dump_mock() {
        test_rpl_slice_dump().unwrap();
    }
}