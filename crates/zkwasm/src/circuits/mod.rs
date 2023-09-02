use self::config::CircuitConfigure;
use crate::checksum::CompilationTableWithParams;
use crate::checksum::ImageCheckSum;
use crate::circuits::config::zkwasm_k;
use crate::circuits::image_table::IMAGE_COL_NAME;
use crate::circuits::utils::Context;

use ark_std::end_timer;
use ark_std::start_timer;
use halo2_proofs::arithmetic::FieldExt;
use halo2_proofs::pairing::bn256::Bn256;
use halo2_proofs::pairing::bn256::Fr;
use halo2_proofs::pairing::bn256::G1Affine;
use halo2_proofs::plonk::create_proof;
use halo2_proofs::plonk::get_advice_commitments_from_transcript;
use halo2_proofs::plonk::keygen_pk;
use halo2_proofs::plonk::keygen_vk;
use halo2_proofs::plonk::verify_proof;
use halo2_proofs::plonk::ConstraintSystem;
use halo2_proofs::plonk::Expression;
use halo2_proofs::plonk::ProvingKey;
use halo2_proofs::plonk::SingleVerifier;
use halo2_proofs::plonk::VerifyingKey;
use halo2_proofs::plonk::VirtualCells;
use halo2_proofs::poly::commitment::Params;
use halo2_proofs::poly::commitment::ParamsVerifier;
use halo2_proofs::transcript::Blake2bRead;
use halo2_proofs::transcript::Blake2bWrite;
use halo2_proofs::transcript::Challenge255;
use num_bigint::BigUint;
use rand::rngs::OsRng;
use specs::CompilationTable;
use specs::Tables;
use std::fs::File;
use std::io::Cursor;
use std::io::Read;
use std::marker::PhantomData;
use std::path::PathBuf;

pub(crate) mod cell;
pub(crate) mod etable;

mod bit_table;
mod external_host_call_table;
mod mtable;
mod traits;

pub mod config;
pub mod image_table;
pub mod jtable;
pub mod rtable;
pub mod test_circuit;
pub mod utils;

#[derive(Default, Clone)]
pub struct TestCircuit<F: FieldExt> {
    pub tables: Tables,
    _data: PhantomData<F>,
}

impl<F: FieldExt> TestCircuit<F> {
    pub fn new(tables: Tables) -> Self {
        CircuitConfigure::from(&tables.compilation_tables).set_global_CIRCUIT_CONFIGURE();

        TestCircuit {
            tables,
            _data: PhantomData,
        }
    }
}

trait Encode {
    fn encode(&self) -> BigUint;
}

pub(self) trait Lookup<F: FieldExt> {
    fn encode(&self, meta: &mut VirtualCells<'_, F>) -> Expression<F>;

    fn configure_in_table(
        &self,
        meta: &mut ConstraintSystem<F>,
        key: &'static str,
        expr: impl FnOnce(&mut VirtualCells<'_, F>) -> Expression<F>,
    ) {
        meta.lookup_any(key, |meta| vec![(expr(meta), self.encode(meta))]);
    }
}

pub struct ZkWasmCircuitBuilder {
    pub tables: Tables,
    pub public_inputs_and_outputs: Vec<u64>,
}

impl ZkWasmCircuitBuilder {
    pub fn build_circuit<F: FieldExt>(&self) -> TestCircuit<F> {
        TestCircuit::new(self.tables.clone())
    }

    // Delete all the following functions, use Loader for bench test.
    fn prepare_param(&self) -> Params<G1Affine> {
        let path = PathBuf::from(format!("test_param.{}.data", zkwasm_k()));

        if path.exists() {
            let mut fd = File::open(path.as_path()).unwrap();
            let mut buf = vec![];

            fd.read_to_end(&mut buf).unwrap();
            Params::<G1Affine>::read(Cursor::new(buf)).unwrap()
        } else {
            // Initialize the polynomial commitment parameters
            let timer = start_timer!(|| format!("build params with K = {}", zkwasm_k()));
            let params: Params<G1Affine> = Params::<G1Affine>::unsafe_setup::<Bn256>(zkwasm_k());
            end_timer!(timer);

            let mut fd = File::create(path.as_path()).unwrap();
            params.write(&mut fd).unwrap();

            params
        }
    }

    fn prepare_vk(
        &self,
        circuit: &TestCircuit<Fr>,
        params: &Params<G1Affine>,
    ) -> VerifyingKey<G1Affine> {
        let timer = start_timer!(|| "build vk");
        let vk = keygen_vk(params, circuit).expect("keygen_vk should not fail");
        end_timer!(timer);

        vk
    }

    fn prepare_pk(
        &self,
        circuit: &TestCircuit<Fr>,
        params: &Params<G1Affine>,
        vk: VerifyingKey<G1Affine>,
    ) -> ProvingKey<G1Affine> {
        let timer = start_timer!(|| "build pk");
        let pk = keygen_pk(&params, vk, circuit).expect("keygen_pk should not fail");
        end_timer!(timer);
        pk
    }

    fn create_proof(
        &self,
        circuits: &[TestCircuit<Fr>],
        params: &Params<G1Affine>,
        pk: &ProvingKey<G1Affine>,
        instance: &Vec<Fr>,
    ) -> Vec<u8> {
        let mut transcript = Blake2bWrite::<_, _, Challenge255<_>>::init(vec![]);

        let timer = start_timer!(|| "create proof");
        create_proof(params, pk, circuits, &[&[instance]], OsRng, &mut transcript)
            .expect("proof generation should not fail");
        end_timer!(timer);

        transcript.finalize()
    }

    fn verify_check(
        &self,
        compile_table: &CompilationTable,
        vk: &VerifyingKey<G1Affine>,
        params: &Params<G1Affine>,
        proof: &Vec<u8>,
        instance: &Vec<Fr>,
    ) {
        let public_inputs_size = instance.len();

        let params_verifier: ParamsVerifier<Bn256> = params.verifier(public_inputs_size).unwrap();

        let strategy = SingleVerifier::new(&params_verifier);
        let mut transcript = Blake2bRead::<_, _, Challenge255<_>>::init(&proof[..]);

        let timer = start_timer!(|| "verify proof");
        verify_proof(
            &params_verifier,
            vk,
            strategy,
            &[&[instance]],
            &mut transcript.clone(),
        )
        .unwrap();
        end_timer!(timer);

        {
            let table = CompilationTableWithParams {
                table: compile_table,
                params,
            };
            let checksum = table.checksum();
            let img_col_idx = vk
                .cs
                .named_advices
                .iter()
                .find(|(k, _)| k == IMAGE_COL_NAME)
                .unwrap()
                .1;
            let img_col_commitment: Vec<G1Affine> =
                get_advice_commitments_from_transcript::<Bn256, _, _>(vk, &mut transcript).unwrap();

            assert!(vec![img_col_commitment[img_col_idx as usize]] == checksum)
        }
    }

    pub fn bench(&self) {
        let mut instances = vec![];

        instances.append(
            &mut self
                .public_inputs_and_outputs
                .iter()
                .map(|v| (*v).into())
                .collect(),
        );

        let circuit: TestCircuit<Fr> = self.build_circuit::<Fr>();

        let params = self.prepare_param();

        let vk = self.prepare_vk(&circuit, &params);
        let pk = self.prepare_pk(&circuit, &params, vk);

        let compile_table = circuit.tables.compilation_tables.clone();

        let proof = self.create_proof(&[circuit], &params, &pk, &instances);

        self.verify_check(&compile_table, pk.get_vk(), &params, &proof, &instances);
    }
}
