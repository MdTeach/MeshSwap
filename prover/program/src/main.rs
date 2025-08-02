#![no_main]
sp1_zkvm::entrypoint!(main);

pub fn main() {
    let secret_key = sp1_zkvm::io::read::<String>();
    let public_params = proofimpl_atomic_swap::make_process(&secret_key);
    sp1_zkvm::io::commit(&public_params);
}
