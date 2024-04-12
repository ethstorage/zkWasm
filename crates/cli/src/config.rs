use std::fs::File;
use std::io::Cursor;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

use anyhow::Result;
use circuits_batcher::args::OpenSchema;
use circuits_batcher::proof::ProofInfo;
use circuits_batcher::proof::ProofLoadInfo;
use circuits_batcher::proof::ProofPieceInfo;
use console::style;
use delphinus_zkwasm::loader::ZkWasmLoader;
use delphinus_zkwasm::runtime::host::HostEnvArg;
use delphinus_zkwasm::runtime::host::HostEnvBuilder;
use halo2_proofs::pairing::bn256::Bn256;
use halo2_proofs::pairing::bn256::G1Affine;
use halo2_proofs::plonk::CircuitData;
use halo2_proofs::poly::commitment::Params;
use indicatif::ProgressBar;
use serde::Deserialize;
use serde::Serialize;
use specs::TraceBackend;

use crate::args::HostMode;
use crate::names::name_of_circuit_data;
use crate::names::name_of_etable_slice;
use crate::names::name_of_instance;
use crate::names::name_of_loadinfo;
use crate::names::name_of_params;
use crate::names::name_of_transcript;
use crate::names::name_of_witness;

#[derive(Serialize, Deserialize)]
pub(crate) struct CircuitDataMd5 {
    pub(crate) circuit_data_md5: String,
    pub(crate) verifying_key_md5: String,
}

#[cfg(not(feature = "continuation"))]
#[derive(Serialize, Deserialize)]
pub(crate) struct CircuitDataConfig {
    pub(crate) finalized_circuit: CircuitDataMd5,
}

#[cfg(feature = "continuation")]
#[derive(Serialize, Deserialize)]
pub(crate) struct CircuitDataConfig {
    pub(crate) on_going_circuit: CircuitDataMd5,
    pub(crate) finalized_circuit: CircuitDataMd5,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Config {
    pub(crate) name: String,

    pub(crate) is_uniform_circuit: bool,
    pub(crate) k: u32,
    pub(crate) params: PathBuf,
    pub(crate) params_md5: String,
    pub(crate) wasm_image_md5: Option<String>,
    pub(crate) circuit_datas: CircuitDataConfig,

    pub(crate) checksum: (String, String),
    pub(crate) phantom_functions: Vec<String>,
    pub(crate) host_mode: HostMode,
}

impl Config {
    fn image_consistent_check(&self, wasm_image: &[u8]) -> anyhow::Result<()> {
        if let Some(expected_wasm_image_md5) = &self.wasm_image_md5 {
            let wasm_image_md5 = format!("{:x}", md5::compute(wasm_image));

            if expected_wasm_image_md5 != &wasm_image_md5 {
                anyhow::bail!(
                    "Wasm image is inconsistent with the one used to build the circuit. \
                        Maybe you have changed the Wasm image after setup the circuit?",
                );
            }
        }

        Ok(())
    }

    fn params_consistent_check(&self, params: &[u8]) -> anyhow::Result<()> {
        let params_md5 = format!("{:x}", md5::compute(params));

        if params_md5 != self.params_md5 {
            anyhow::bail!(
                "Params is inconsistent with the one used to build the circuit. \
                    Maybe you have changed the params after setup the circuit?",
            );
        }

        Ok(())
    }

    fn veryfying_key_consistent_check(
        &self,
        verifying_key: &[u8],
        expected_md5: &str,
    ) -> anyhow::Result<()> {
        let verifying_key_md5 = format!("{:x}", md5::compute(&verifying_key));

        if verifying_key_md5 != expected_md5 {
            anyhow::bail!(
                "Verifying key is inconsistent with the one used to build the circuit. \
                    Maybe you have changed the circuit data after setup the circuit?",
            );
        }

        Ok(())
    }
}

impl Config {
    pub(crate) fn write(&self, fd: &mut File) -> anyhow::Result<()> {
        fd.write(&bincode::serialize(self)?)?;

        Ok(())
    }

    pub(crate) fn read(fd: &mut File) -> anyhow::Result<Self> {
        let mut buf = Vec::new();
        fd.read_to_end(&mut buf)?;
        let config = bincode::deserialize(&mut buf)?;

        Ok(config)
    }
}

impl Config {
    fn read_wasm_image(&self, wasm_image: &PathBuf) -> anyhow::Result<Vec<u8>> {
        let mut buf = Vec::new();
        File::open(&wasm_image)?.read_to_end(&mut buf)?;

        self.image_consistent_check(&buf)?;

        Ok(buf)
    }

    fn read_params(&self, params_dir: &PathBuf) -> anyhow::Result<Params<G1Affine>> {
        let path = params_dir.join(name_of_params(self.k));

        let mut buf = Vec::new();
        File::open(&path)?.read_to_end(&mut buf)?;

        self.params_consistent_check(&buf)?;

        let params = Params::<G1Affine>::read(&mut Cursor::new(&mut buf))?;

        Ok(params)
    }

    fn read_circuit_data(
        &self,
        path: &PathBuf,
        expected_md5: &str,
    ) -> anyhow::Result<CircuitData<G1Affine>> {
        let mut buf = Vec::new();
        File::open(&path)?.read_to_end(&mut buf)?;

        let circuit_data_md5 = format!("{:x}", md5::compute(&buf));

        if circuit_data_md5 != expected_md5 {
            anyhow::bail!(
                "Circuit data is inconsistent with the one used to build the circuit. \
                    Maybe you have changed the circuit data after setup the circuit?",
            );
        }

        let circuit_data = CircuitData::<G1Affine>::read(&mut File::open(&path)?)?;

        Ok(circuit_data)
    }

    pub(crate) fn dry_run<EnvBuilder: HostEnvBuilder>(
        self,
        wasm_image: &PathBuf,
        output_dir: &PathBuf,
        arg: EnvBuilder::Arg,
        context_output_filename: Option<String>,
    ) -> Result<()> {
        let wasm_image = self.read_wasm_image(wasm_image)?;

        let context_output = arg.get_context_output();

        let loader = ZkWasmLoader::<Bn256, EnvBuilder::Arg, EnvBuilder>::new(
            self.k,
            wasm_image,
            self.phantom_functions,
        )?;

        {
            println!("{} Executing...", style("[1/2]").bold().dim(),);
            let result = loader.run(
                arg,
                EnvBuilder::HostConfig::default(),
                true,
                TraceBackend::Memory,
            )?;

            println!("total guest instructions used {:?}", result.guest_statics);
            println!("total host api used {:?}", result.host_statics);
        };

        {
            if let Some(context_output_filename) = context_output_filename {
                let context_output_path = output_dir.join(context_output_filename);

                println!(
                    "{} Write context output to file {:?}...",
                    style("[2/2]").bold().dim(),
                    context_output_path
                );

                context_output.write(&mut File::create(&context_output_path)?)?;
            } else {
                println!(
                    "{} Context output is not specified. Skip writing context output...",
                    style("[2/2]").bold().dim()
                );
            }
        }

        Ok(())
    }

    pub(crate) fn prove<EnvBuilder: HostEnvBuilder>(
        self,
        wasm_image: &PathBuf,
        params_dir: &PathBuf,
        output_dir: &PathBuf,
        arg: EnvBuilder::Arg,
        context_output_filename: Option<String>,
        mock_test: bool,
        backend: TraceBackend,
    ) -> anyhow::Result<()> {
        println!("{} Load image...", style("[1/8]").bold().dim(),);
        let wasm_image = self.read_wasm_image(wasm_image)?;

        println!("{} Load params...", style("[2/8]").bold().dim(),);
        let params = self.read_params(params_dir)?;

        let loader = ZkWasmLoader::<Bn256, EnvBuilder::Arg, EnvBuilder>::new(
            self.k,
            wasm_image,
            self.phantom_functions.clone(),
        )?;

        let context_output = arg.get_context_output();

        let result = {
            println!("{} Executing...", style("[3/8]").bold().dim(),);
            let result = loader.run(arg, EnvBuilder::HostConfig::default(), false, backend)?;

            println!("total guest instructions used {:?}", result.guest_statics);
            println!("total host api used {:?}", result.host_statics);

            result
        };

        {
            if let Some(context_output_filename) = context_output_filename {
                let context_output_path = output_dir.join(context_output_filename);

                println!(
                    "{} Write context output to file {:?}...",
                    style("[4/8]").bold().dim(),
                    context_output_path
                );

                context_output.write(&mut File::create(&context_output_path)?)?;
            } else {
                println!(
                    "{} Context output is not specified. Skip writing context output...",
                    style("[4/8]").bold().dim()
                );
            }
        }

        {
            let dir = output_dir.join("traces");

            println!(
                "{} Writing traces to {:?}...",
                style("[5/8]").bold().dim(),
                dir
            );
            result
                .tables
                .write(&dir, |slice| name_of_etable_slice(&self.name, slice));
        }

        println!("{} Build circuit(s)...", style("[6/8]").bold().dim(),);
        let instances = result
            .public_inputs_and_outputs
            .clone()
            .iter()
            .map(|v| (*v).into())
            .collect::<Vec<_>>();

        println!("{} Creating proof(s)...", style("[7/8]").bold().dim(),);

        let mut proof_load_info = ProofLoadInfo::new(
            &self.name,
            self.k as usize,
            circuits_batcher::args::HashType::Poseidon,
        );

        let progress_bar = ProgressBar::new(result.tables.execution_tables.etable.len() as u64);

        let mut slices = loader.slice(result)?.into_iter().enumerate().peekable();
        while let Some((index, circuit)) = slices.next() {
            let circuit = circuit?;

            let _is_finalized_circuit = slices.peek().is_none();

            if mock_test {
                println!("mock test for slice {}...", index);
                loader.mock_test(&circuit, &instances)?;
            }

            #[cfg(feature = "continuation")]
            let proving_key = if _is_finalized_circuit {
                self.read_circuit_data(
                    &params_dir.join(name_of_circuit_data(&self.name, true)),
                    &self.circuit_datas.finalized_circuit.circuit_data_md5,
                )?
                .into_proving_key(&params)
            } else {
                self.read_circuit_data(
                    &params_dir.join(name_of_circuit_data(&self.name, false)),
                    &self.circuit_datas.on_going_circuit.circuit_data_md5,
                )?
                .into_proving_key(&params)
            };

            #[cfg(not(feature = "continuation"))]
            let proving_key = self
                .read_circuit_data(
                    &params_dir.join(name_of_circuit_data(&self.name, true)),
                    &self.circuit_datas.finalized_circuit.circuit_data_md5,
                )?
                .into_proving_key(&params);

            let circuit_data_name = name_of_circuit_data(&self.name, _is_finalized_circuit);

            let proof_piece_info = ProofPieceInfo {
                circuit: circuit_data_name,
                instance_size: instances.len() as u32,
                witness: name_of_witness(&self.name, index),
                instance: name_of_instance(&self.name, index),
                transcript: name_of_transcript(&self.name, index),
            };

            proof_piece_info.exec_create_proof_with_params::<Bn256, _>(
                &circuit,
                &vec![instances.clone()],
                &params,
                &proving_key,
                output_dir,
                proof_load_info.hashtype,
                OpenSchema::Shplonk,
            );

            proof_load_info.append_single_proof(proof_piece_info);

            progress_bar.inc(1);
        }
        progress_bar.finish_and_clear();

        {
            let proof_load_info_path = output_dir.join(&name_of_loadinfo(&self.name));
            println!(
                "{} Saving proof load info to {:?}...",
                style("[8/8]").bold().dim(),
                proof_load_info_path
            );
            proof_load_info.save(proof_load_info_path.parent().unwrap());
        }

        Ok(())
    }

    pub(crate) fn verify<EnvBuilder: HostEnvBuilder>(
        self,
        params_dir: &PathBuf,
        output_dir: &PathBuf,
    ) -> anyhow::Result<()> {
        let mut proofs = {
            println!(
                "{} Reading proofs from {:?}",
                style("[1/2]").bold().dim(),
                output_dir
            );

            let proof_load_info =
                ProofLoadInfo::load(&output_dir.join(&name_of_loadinfo(&self.name)));

            let proofs: Vec<ProofInfo<Bn256>> =
                ProofInfo::load_proof(&output_dir, &params_dir, &proof_load_info);

            proofs
        }
        .into_iter()
        .peekable();

        println!(
            "{} Found {} proofs, verifying..",
            style("[2/2]").bold().dim(),
            proofs.len()
        );

        let progress_bar = ProgressBar::new(proofs.len() as u64);
        while let Some(proof) = proofs.next() {
            let params_verifier = {
                let public_inputs_size = proof
                    .instances
                    .iter()
                    .fold(0, |acc, x| usize::max(acc, x.len()));

                let params = self.read_params(params_dir)?;
                params.verifier(public_inputs_size)?
            };

            {
                let mut buf = Vec::new();
                proof.vkey.write(&mut Cursor::new(&mut buf))?;

                #[cfg(feature = "continuation")]
                if proofs.peek().is_none() {
                    self.veryfying_key_consistent_check(
                        &buf,
                        &self.circuit_datas.finalized_circuit.verifying_key_md5,
                    )?;
                } else {
                    self.veryfying_key_consistent_check(
                        &buf,
                        &self.circuit_datas.on_going_circuit.verifying_key_md5,
                    )?;
                }

                #[cfg(not(feature = "continuation"))]
                self.veryfying_key_consistent_check(
                    &buf,
                    &self.circuit_datas.finalized_circuit.verifying_key_md5,
                )?;
            };

            proof.verify_proof(&params_verifier).unwrap();

            // TODO: handle checksum sanity check
            // #[cfg(feature = "uniform-circuit")]
            // {
            //     use delphinus_zkwasm::circuits::image_table::IMAGE_COL_NAME;
            //     use halo2_proofs::plonk::get_advice_commitments_from_transcript;
            //     use halo2aggregator_s::transcript::poseidon::PoseidonRead;

            //     let _img_col_idx = proof
            //         .vkey
            //         .cs
            //         .named_advices
            //         .iter()
            //         .find(|(k, _)| k == IMAGE_COL_NAME)
            //         .unwrap()
            //         .1;
            //     let _img_col_commitment: Vec<G1Affine> =
            //         get_advice_commitments_from_transcript::<Bn256, _, _>(
            //             &proof.vkey,
            //             &mut PoseidonRead::init(&proof.transcripts[..]),
            //         )
            //         .unwrap();

            //     assert!(
            //         vec![_img_col_commitment[_img_col_idx as usize]][0]
            //             .x
            //             .to_string()
            //             == self.checksum.0
            //     );
            //     assert!(
            //         vec![_img_col_commitment[_img_col_idx as usize]][0]
            //             .y
            //             .to_string()
            //             == self.checksum.1
            //     );
            // }

            progress_bar.inc(1);
        }
        progress_bar.finish_and_clear();

        println!("{}", style("Verification succeeded!").green().bold().dim(),);

        Ok(())
    }
}
