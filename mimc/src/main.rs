use plonky2::{field::goldilocks_field::GoldilocksField, plonk::config::PoseidonGoldilocksConfig};

use zk_lib::hashes::mimc::mimc::MiMC;

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
    let mimc = MiMC::<GoldilocksField>::new_from_rng();
    let hash = mimc.permute_rounds([GoldilocksField(1), GoldilocksField(2)]);
    println!("Hash: {}", hash);
    type C = PoseidonGoldilocksConfig;

    let data;
    let pw;

    // mimc circuit
    let mut timing = TimingTree::new("generate pw", Level::Info);
    timed!(timing, "witness generation", {
        (data, pw) = mimc.circuit_generation::<C, 2>([GoldilocksField(1), GoldilocksField(2)]);
    });
    timing.print();

    // prove
    let timing = TimingTree::new("prove", Level::Debug);
    let proof = MiMC::proof_generation(&data, &pw);
    timing.print();

    // verify
    let timing = TimingTree::new("verify", Level::Debug);
    MiMC::proof_verification(&data, &proof);
    timing.print();

    Ok(data.verify(proof)?)
}
