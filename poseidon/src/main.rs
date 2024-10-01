use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::field::types::Sample;
use plonky2::gates::gate::Gate;
use plonky2::hash::hash_types::NUM_HASH_OUT_ELTS;

use plonky2::iop::witness::PartialWitness;
use plonky2::iop::witness::WitnessWrite;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;

use std::cmp;
use std::error::Error;

use log::{Level, LevelFilter};
use plonky2::gates::poseidon::PoseidonGate;
use plonky2::hash::poseidon::PoseidonHash;
use plonky2::plonk::config::PoseidonGoldilocksConfig;
use plonky2::plonk::prover::prove;
use plonky2::timed;
use plonky2::util::timing::TimingTree;

const D: usize = 2;
type F = GoldilocksField;

fn generate_config_for_poseidon() -> CircuitConfig {
    let needed_wires = cmp::max(
        PoseidonGate::<F, D>::new().num_wires(),
        CircuitConfig::standard_recursion_config().num_wires,
    );
    CircuitConfig {
        num_wires: needed_wires,
        num_routed_wires: needed_wires,
        ..CircuitConfig::standard_recursion_config()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    let mut logger = env_logger::Builder::from_default_env();
    logger.format_timestamp(None);
    logger.filter_level(LevelFilter::Debug);
    logger.try_init()?;

    let config = generate_config_for_poseidon();

    let mut builder = CircuitBuilder::<F, D>::new(config);
    let inp_targets_array = builder.add_virtual_target_arr::<{ NUM_HASH_OUT_ELTS }>();
    let mut res_targets_array = inp_targets_array.clone();
    for _ in 0..100 {
        res_targets_array = builder
            .hash_or_noop::<PoseidonHash>(res_targets_array.to_vec())
            .elements;
    }
    builder.register_public_inputs(&res_targets_array);

    let mut pw = PartialWitness::<F>::new();

    let mut timing = TimingTree::new("generate pw", Level::Info);
    timed!(timing, "witness generation", {
        inp_targets_array.into_iter().for_each(|t| {
            let input = F::rand();
            pw.set_target(t, input);
        });
    });
    timing.print();

    // print gates number
    println!("Constructing proof with {} gates", builder.num_gates(),);

    let data = builder.build::<PoseidonGoldilocksConfig>();

    // prove

    let mut timing = TimingTree::new("prove", Level::Debug);

    let original_proof = prove(&data.prover_only, &data.common, pw, &mut timing).unwrap();
    timing.print();

    let timing = TimingTree::new("verify", Level::Debug);
    data.verify(original_proof.clone()).unwrap();
    timing.print();

    println!("Proof size: {}", original_proof.to_bytes().len(),);

    // let proof = data.prove(pw)?;

    Ok(data.verify(original_proof)?)
}
