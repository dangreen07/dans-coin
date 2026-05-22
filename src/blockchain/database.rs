use crate::transactions::Recipient;
use crate::transactions::TransactionData;
use crate::wallet::Wallet;
use rusqlite::Connection;
use rusqlite::params;

use crate::{
    blockchain::{Block, BlockData, HashableBlock},
    transactions::Transaction,
};

pub struct BlockChain {
    connection: Connection,
}

impl BlockChain {
    pub fn new() -> Self {
        let connection = Connection::open("blockchain.sqlite").unwrap();
        Self::initialize_database(&connection);

        BlockChain {
            connection: connection,
        }
    }

    pub fn length(&self) -> usize {
        let connection = &self.connection;
        let mut stmt = connection.prepare("SELECT COUNT(*) FROM blocks").unwrap();
        let length: usize = stmt
            .query_row([], |row| {
                let length: i64 = row.get(0).unwrap();
                Ok(length as usize)
            })
            .unwrap();
        return length;
    }

    fn initialize_database(connection: &Connection) {
        connection
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS blocks (
id INTEGER PRIMARY KEY,
hash BLOB NOT NULL,
nonce INTEGER NOT NULL,
previous_hash BLOB NOT NULL
);

CREATE TABLE IF NOT EXISTS transactions (
hash_id BLOB PRIMARY KEY,
sender_address BLOB NOT NULL,
input_amount FLOAT NOT NULL,
fee FLOAT NOT NULL,
timestamp BLOB NOT NULL,
block_id INTEGER NOT NULL,
FOREIGN KEY(block_id) REFERENCES blocks(id)
);

CREATE TABLE IF NOT EXISTS recipients (
address BLOB NOT NULL,
amount FLOAT NOT NULL,
transaction_id BLOB,
FOREIGN KEY(transaction_id) REFERENCES transactions(hash_id)
);",
            )
            .unwrap();
    }

    pub fn add_block(&self, block: Block) -> Result<Block, rusqlite::Error> {
        let connection = &self.connection;
        let result = connection.execute(
            "
    INSERT INTO blocks (id, hash, nonce, previous_hash) VALUES (?1, ?2, ?3, ?4)",
            (
                block.hashable_block.block.id as i64,
                block.hash,
                block.hashable_block.nonce as i64,
                block.hashable_block.block.previous_hash,
            ),
        );
        if result.is_err() {
            return Err(result.err().unwrap());
        }
        for transaction in block.hashable_block.block.transactions.iter() {
            let result = transaction.add_to_database(&connection, &block);
            if result.is_err() {
                return Err(result.err().unwrap());
            }
        }
        for transaction in block.hashable_block.block.transactions.iter() {
            for recipient in transaction.transaction_data.recipients.iter() {
                let result = recipient.add_to_database(&connection, transaction);
                if result.is_err() {
                    return Err(result.err().unwrap());
                }
            }
        }
        return Ok(block);
    }

    pub fn get_block(&self, id: u64) -> Result<Block, rusqlite::Error> {
        let connection = &self.connection;
        let stmt = connection.prepare("SELECT * FROM blocks WHERE id = ?1");
        let mut stmt = match stmt {
            Ok(stmt) => stmt,
            Err(error) => {
                println!("Error getting block: {}", error);
                return Err(error);
            }
        };
        let mut block = stmt
            .query_one(params![id as i64], |row| {
                let hash: [u8; 64] = row.get(1).unwrap();
                let nonce: i64 = row.get(2).unwrap();
                let previous_hash: [u8; 64] = row.get(3).unwrap();
                Ok(Block {
                    hashable_block: HashableBlock {
                        block: BlockData {
                            id: id,
                            transactions: Vec::new(),
                            previous_hash: previous_hash,
                        },
                        nonce: nonce as u64,
                    },
                    hash: hash,
                })
            })
            .unwrap();
        let mut stmt = connection
            .prepare("SELECT * FROM transactions WHERE block_id = ?1")
            .unwrap();
        let mut transactions = stmt
            .query_map(params![id as i64], |row| {
                let hash_id: [u8; 64] = row.get(0).unwrap();
                let sender_address: [u8; 32] = row.get(1).unwrap();
                let input_amount: f64 = row.get(2).unwrap();
                let fee: f64 = row.get(3).unwrap();
                let timestamp: u128 = u128::from_le_bytes(row.get(4).unwrap());
                let mut transaction = Transaction {
                    transaction_data: TransactionData {
                        sender_address: sender_address,
                        input_amount: input_amount,
                        fee: fee,
                        recipients: Vec::new(),
                        timestamp: timestamp,
                    },
                    hash: hash_id,
                    signature: [0; 64],
                };
                // Recipients
                let mut stmt = connection
                    .prepare("SELECT * FROM recipients WHERE transaction_id = ?1")
                    .unwrap();
                let mut recipients = stmt
                    .query_map(params![transaction.hash], |row| {
                        let address: [u8; 32] = row.get(0).unwrap();
                        let amount: f64 = row.get(1).unwrap();
                        Ok(Recipient::new(address, amount))
                    })
                    .unwrap();
                while let Some(recipient) = recipients.next() {
                    if let Ok(recipient) = recipient {
                        transaction.transaction_data.recipients.push(recipient);
                    }
                }

                Ok(transaction)
            })
            .unwrap();
        while let Some(transaction) = transactions.next() {
            if let Ok(transaction) = transaction {
                block.hashable_block.block.transactions.push(transaction);
            }
        }
        return Ok(block);
    }

    pub fn list_wallet_transactions(&self, wallet: &Wallet) -> Vec<Transaction> {
        let connection = &self.connection;
        wallet.list_transactions(connection)
    }

    pub fn get_latest_block(&self) -> Result<Block, rusqlite::Error> {
        let connection = &self.connection;
        let mut stmt = connection
            .prepare("SELECT * FROM blocks ORDER BY id DESC LIMIT 1")
            .unwrap();
        let mut block = stmt
            .query_one(params![], |row| {
                let hash: [u8; 64] = row.get(1).unwrap();
                let nonce: i64 = row.get(2).unwrap();
                let previous_hash: [u8; 64] = row.get(3).unwrap();
                let id: i64 = row.get(0).unwrap();
                Ok(Block {
                    hashable_block: HashableBlock {
                        block: BlockData {
                            id: id as u64,
                            transactions: Vec::new(),
                            previous_hash: previous_hash,
                        },
                        nonce: nonce as u64,
                    },
                    hash: hash,
                })
            })
            .unwrap();
        let mut stmt = connection
            .prepare("SELECT * FROM transactions WHERE block_id = ?1")
            .unwrap();
        let mut transactions = stmt
            .query_map(params![block.hashable_block.block.id as i64], |row| {
                let hash_id: [u8; 64] = row.get(0).unwrap();
                let sender_address: [u8; 32] = row.get(1).unwrap();
                let input_amount: f64 = row.get(2).unwrap();
                let fee: f64 = row.get(3).unwrap();
                let timestamp: u128 = u128::from_le_bytes(row.get(4).unwrap());
                let mut transaction = Transaction {
                    transaction_data: TransactionData {
                        sender_address: sender_address,
                        input_amount: input_amount,
                        fee: fee,
                        recipients: Vec::new(),
                        timestamp: timestamp,
                    },
                    hash: hash_id,
                    signature: [0; 64],
                };
                // Recipients
                let mut stmt = connection
                    .prepare("SELECT * FROM recipients WHERE transaction_id = ?1")
                    .unwrap();
                let mut recipients = stmt
                    .query_map(params![transaction.hash], |row| {
                        let address: [u8; 32] = row.get(0).unwrap();
                        let amount: f64 = row.get(1).unwrap();
                        Ok(Recipient::new(address, amount))
                    })
                    .unwrap();
                while let Some(recipient) = recipients.next() {
                    if let Ok(recipient) = recipient {
                        transaction.transaction_data.recipients.push(recipient);
                    }
                }

                Ok(transaction)
            })
            .unwrap();
        while let Some(transaction) = transactions.next() {
            if let Ok(transaction) = transaction {
                block.hashable_block.block.transactions.push(transaction);
            }
        }
        return Ok(block);
    }
}
