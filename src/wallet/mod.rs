use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::RngCore;
use serde::{Deserialize, Serialize};

pub struct Wallet {
    pub secret_key: SigningKey,
    pub public_key: VerifyingKey,
}

impl Wallet {
    pub fn new() -> Self {
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

    pub fn save_wallet(&self) {
        let saveable_wallet = self.to_saveable();
        let saveable_wallet_json = serde_json::to_string(&saveable_wallet).unwrap();
        std::fs::write("wallet.json", saveable_wallet_json).unwrap();
    }

    pub fn load_wallet() -> Result<Self, &'static str> {
        let saveable_wallet_json = std::fs::read_to_string("wallet.json");
        if let Ok(saveable_wallet_json) = saveable_wallet_json {
            let saveable_wallet: Result<SaveableWallet, serde_json::Error> =
                serde_json::from_str(&saveable_wallet_json);
            if let Ok(saveable_wallet) = saveable_wallet {
                return Ok(saveable_wallet.from_saveable());
            }
            return Err("Error loading wallet");
        }
        return Err("Wallet not found");
    }

    fn to_saveable(&self) -> SaveableWallet {
        SaveableWallet {
            secret_key: hex::encode(self.secret_key.to_bytes()),
            public_key: hex::encode(self.public_key.to_bytes()),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SaveableWallet {
    pub secret_key: String,
    pub public_key: String,
}

impl SaveableWallet {
    pub fn from_saveable(&self) -> Wallet {
        let secret_key = hex::decode(self.secret_key.clone())
            .unwrap()
            .as_slice()
            .try_into()
            .unwrap();
        let public_key = hex::decode(self.public_key.clone())
            .unwrap()
            .as_slice()
            .try_into()
            .unwrap();
        Wallet {
            secret_key: SigningKey::from_bytes(&secret_key),
            public_key: VerifyingKey::from_bytes(&public_key).unwrap(),
        }
    }
}
