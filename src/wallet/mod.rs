use crate::transactions::Recipient;
use crate::transactions::Transaction;
use crate::transactions::TransactionData;
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::RngCore;
use rusqlite::Connection;
use rusqlite::params;
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

    pub fn list_transactions(&self, connection: &Connection) -> Vec<Transaction> {
        let mut transactions = Vec::new();
        let mut stmt = connection
            .prepare("SELECT * FROM recipients WHERE address = ?1")
            .unwrap();
        let mut recieved_transactions = stmt
            .query_map(params![self.public_key.to_bytes()], |row| {
                let address: [u8; 32] = row.get(0).unwrap();
                let amount: f64 = row.get(1).unwrap();
                let hash_id: [u8; 64] = row.get(2).unwrap();
                let recipient = Recipient::new(address, amount);
                let mut stmt = connection
                    .prepare("SELECT * FROM transactions WHERE hash_id = ?1")
                    .unwrap();
                let transaction = stmt
                    .query_one(params![hash_id], |row| {
                        let sender_address: [u8; 32] = row.get(1).unwrap();
                        let input_amount: f64 = row.get(2).unwrap();
                        let fee: f64 = row.get(3).unwrap();
                        let timestamp: u128 = u128::from_le_bytes(row.get(4).unwrap());
                        let transaction = Transaction {
                            transaction_data: TransactionData {
                                sender_address: sender_address,
                                input_amount: input_amount,
                                fee: fee,
                                recipients: vec![recipient],
                                timestamp: timestamp,
                            },
                            hash: hash_id,
                            signature: [0; 64],
                        };
                        Ok(transaction)
                    })
                    .unwrap();
                Ok(transaction)
            })
            .unwrap();
        while let Some(transaction) = recieved_transactions.next() {
            if let Ok(transaction) = transaction {
                transactions.push(transaction);
            }
        }
        transactions
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
