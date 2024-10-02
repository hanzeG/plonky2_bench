use plonky2::{
    field::{goldilocks_field::GoldilocksField, types::Field},
    plonk::config::PoseidonGoldilocksConfig,
};

use zk_lib::hashes::arion::{arion::Arion, SPONGE_RATE};

use log::{Level, LevelFilter};
use plonky2::timed;
use plonky2::util::timing::TimingTree;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    let mut logger = env_logger::Builder::from_default_env();
    logger.format_timestamp(None);
    logger.filter_level(LevelFilter::Debug);
    logger.try_init()?;

    // input
    let mut input = [GoldilocksField::ZERO; SPONGE_RATE];
    for i in 0..SPONGE_RATE {
        input[i] = GoldilocksField(i as u64);
    }

    // Arion Hash
    let output = Arion::arion_hash::<GoldilocksField, 4>(input.clone());
    for i in 0..output.len() {
        println!("Output {}: {}", i, output[i]);
    }

    let data;
    let pw;

    // Arion circuit
    let mut timing = TimingTree::new("generate pw", Level::Info);
    timed!(timing, "witness generation", {
        (data, pw) =
            Arion::circuit_generation::<GoldilocksField, PoseidonGoldilocksConfig, SPONGE_RATE>(
                input.clone(),
            );
    });
    timing.print();

    // prove
    let timing = TimingTree::new("prove", Level::Debug);
    let proof = Arion::proof_generation::<GoldilocksField, PoseidonGoldilocksConfig, 2>(&data, &pw);
    timing.print();

    // verify
    let timing = TimingTree::new("verify", Level::Debug);
    Arion::proof_verification::<GoldilocksField, PoseidonGoldilocksConfig, 2>(&data, &proof);
    timing.print();

    Ok(data.verify(proof)?)
}
