use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::RngCore;

pub struct Wallet {
    pub secret_key: SigningKey,
    pub public_key: VerifyingKey,
}

pub fn create_wallet() -> Wallet {
    let mut rng = rand::thread_rng();
    let mut secret_key: [u8; 32] = [0; 32];
    rng.fill_bytes(&mut secret_key);
    let secret_key = SigningKey::from_bytes(&secret_key);
    let public_key = secret_key.verifying_key();
    Wallet {
        secret_key: secret_key,
        public_key: public_key,
    }
}
