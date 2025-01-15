#![no_main]
use committee_iso::utils::{Digest, Sha256};
use step_iso::types::RecursiveInputs;
sp1_zkvm::entrypoint!(main);

pub fn main() {
    let inputs: RecursiveInputs = borsh::from_slice(&sp1_zkvm::io::read_vec()).unwrap();
    sp1_zkvm::lib::verify::verify_sp1_proof(
        &inputs.vk,
        &(Sha256::digest(inputs.public_values.clone())).into(),
    );
    // todo: commit a SOL encoded value instead!
    sp1_zkvm::io::commit(&inputs);
}
