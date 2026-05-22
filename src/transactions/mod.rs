use ed25519_dalek::{Signer, SigningKey};
use rusqlite::Connection;
use serde::Deserialize;
use serde::Serialize;
use serde_with::{Bytes, serde_as};
use sha3::Digest;
use std::time::SystemTime;

use crate::blockchain::Block;

#[derive(Clone, Serialize, Deserialize)]
pub struct Recipient {
    address: [u8; 32],
    amount: f64,
}

impl Recipient {
    pub fn new(address: [u8; 32], amount: f64) -> Self {
        Recipient { address, amount }
    }

    pub fn add_to_database(
        &self,
        connection: &Connection,
        transaction: &Transaction,
    ) -> Result<usize, rusqlite::Error> {
        let mut recipients = connection
            .prepare("INSERT INTO recipients (address, amount, transaction_id) VALUES (?1, ?2, ?3)")
            .unwrap();
        recipients.execute((self.address, self.amount, transaction.hash))
    }

    pub fn display(&self, offset: usize) {
        println!(
            "{}Address : {}",
            " ".repeat(offset),
            hex::encode(self.address)
        );
        println!("{}Amount  : {:.10}", " ".repeat(offset), self.amount);
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TransactionData {
    pub sender_address: [u8; 32],
    pub input_amount: f64,
    pub fee: f64,
    pub recipients: Vec<Recipient>,
    pub timestamp: u128,
}

impl TransactionData {
    pub fn format(&self) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend(self.sender_address);
        result.extend(self.input_amount.to_le_bytes());
        result.extend(self.fee.to_le_bytes());
        result.extend(self.recipients.len().to_le_bytes());
        for recipient in self.recipients.iter() {
            result.extend(recipient.address);
            result.extend(recipient.amount.to_le_bytes());
        }
        result.extend(self.timestamp.to_le_bytes());
        return result;
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let sender_address: [u8; 32] = bytes[0..32].try_into().unwrap();
        let input_amount: f64 = f64::from_le_bytes(bytes[32..40].try_into().unwrap());
        let fee: f64 = f64::from_le_bytes(bytes[40..48].try_into().unwrap());
        let recipients_length: usize = usize::from_le_bytes(bytes[48..56].try_into().unwrap());
        let mut recipients = Vec::new();
        for i in 0..recipients_length {
            let recipient_address: [u8; 32] =
                bytes[56 + i * 32..56 + (i + 1) * 32].try_into().unwrap();
            let recipient_amount: f64 = f64::from_le_bytes(
                bytes[56 + (i + 1) * 32..56 + (i + 2) * 32]
                    .try_into()
                    .unwrap(),
            );
            recipients.push(Recipient {
                address: recipient_address,
                amount: recipient_amount,
            });
        }
        let timestamp: u128 = u128::from_le_bytes(
            bytes[56 + (recipients_length + 1) * 32..56 + (recipients_length + 2) * 32]
                .try_into()
                .unwrap(),
        );
        return TransactionData {
            sender_address: sender_address,
            input_amount: input_amount,
            fee: fee,
            recipients: recipients,
            timestamp: timestamp,
        };
    }
}

#[serde_as]
#[derive(Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub transaction_data: TransactionData,
    #[serde_as(as = "Bytes")]
    pub hash: [u8; 64],
    #[serde_as(as = "Bytes")]
    pub signature: [u8; 64],
}

impl Transaction {
    pub fn add_to_database(
        &self,
        connection: &Connection,
        block: &Block,
    ) -> Result<usize, rusqlite::Error> {
        let mut transactions = connection.prepare("INSERT INTO transactions (hash_id, sender_address, input_amount, fee, timestamp, block_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)").unwrap();
        transactions.execute((
            self.hash,
            self.transaction_data.sender_address,
            self.transaction_data.input_amount,
            self.transaction_data.fee,
            self.transaction_data.timestamp.to_le_bytes(),
            block.hashable_block.block.id as i64,
        ))
    }

    pub fn create_reward_transaction(recipient_address: [u8; 32]) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let reward = 10.0;
        let transaction = TransactionData {
            sender_address: [0; 32],
            input_amount: reward, // 10 coins is the reward for mining a block, plus the fees
            fee: 0.0,
            recipients: vec![Recipient::new(recipient_address, reward)],
            timestamp: timestamp,
        };
        let mut hasher = sha3::Keccak512::new();
        hasher.update(transaction.format());
        let hashed_transaction: [u8; 64] = hasher.finalize().into();
        let signature = [0; 64];
        Transaction {
            transaction_data: transaction,
            hash: hashed_transaction,
            signature: signature,
        }
    }

    pub fn create_transaction(
        sender_address: [u8; 32],
        input_amount: f64,
        fee: f64,
        recipients: Vec<Recipient>,
        secret_key: SigningKey,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let transaction = TransactionData {
            sender_address: sender_address,
            input_amount: input_amount,
            fee: fee,
            recipients: recipients,
            timestamp: timestamp,
        };
        let mut hasher = sha3::Keccak512::new();
        hasher.update(transaction.format());
        let hashed_transaction: [u8; 64] = hasher.finalize().into();
        let signature = secret_key.sign(&hashed_transaction).to_bytes();
        Transaction {
            transaction_data: transaction,
            hash: hashed_transaction,
            signature: signature,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TransactionQueue {
    transactions: Vec<Transaction>,
}

impl TransactionQueue {
    pub fn new() -> Self {
        let transaction_queue_json = std::fs::read_to_string("transactions.json");
        if let Ok(transaction_queue) = transaction_queue_json {
            let transaction_queue: Result<TransactionQueue, serde_json::Error> =
                serde_json::from_str(&transaction_queue);
            if let Ok(transaction_queue) = transaction_queue {
                return transaction_queue;
            }
        }
        TransactionQueue {
            transactions: Vec::new(),
        }
    }

    fn save(&self) {
        let transaction_queue_string = serde_json::to_string(&self).unwrap();
        std::fs::write("transactions.json", transaction_queue_string).unwrap();
    }

    pub fn add(&mut self, transaction: Transaction) {
        // Check the transactions signature and that the input + fee = output total
        self.transactions.push(transaction);
        self.save();
    }

    pub fn get(&self) -> Vec<Transaction> {
        self.transactions.clone()
    }

    pub fn len(&self) -> usize {
        self.transactions.len()
    }
}
