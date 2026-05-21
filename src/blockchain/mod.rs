use crate::transactions::{Transaction, TransactionData};
use sha3::Digest;

pub mod database;

#[derive(Clone)]
pub struct BlockData {
    pub id: u64,
    transactions: Vec<Transaction>,
    previous_hash: [u8; 64],
}

impl BlockData {
    pub fn format(&self) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend(self.id.to_le_bytes());
        result.extend(self.transactions.len().to_le_bytes());
        for transaction in self.transactions.iter() {
            result.extend(transaction.transaction_data.format());
            result.extend(transaction.hash);
            result.extend(transaction.signature);
        }
        result.extend(self.previous_hash);
        return result;
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let index: u64 = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
        let transactions_length: usize = usize::from_le_bytes(bytes[8..16].try_into().unwrap());
        let mut transactions = Vec::new();
        for i in 0..transactions_length {
            let transaction_data =
                TransactionData::from_bytes(&bytes[16 + i * 128..16 + (i + 1) * 128]);
            let hash: [u8; 64] = bytes[16 + (i + 1) * 128..16 + (i + 2) * 128]
                .try_into()
                .unwrap();
            let signature: [u8; 64] = bytes[16 + (i + 2) * 128..16 + (i + 3) * 128]
                .try_into()
                .unwrap();
            transactions.push(Transaction {
                transaction_data: transaction_data,
                hash: hash,
                signature: signature,
            });
        }
        let previous_hash: [u8; 64] = bytes
            [16 + (transactions_length + 1) * 128..16 + (transactions_length + 2) * 128]
            .try_into()
            .unwrap();
        return BlockData {
            id: index,
            transactions: transactions,
            previous_hash: previous_hash,
        };
    }
}

#[derive(Clone)]
pub struct HashableBlock {
    pub block: BlockData,
    nonce: u64,
}

impl HashableBlock {
    pub fn format(&self) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend(self.block.format());
        result.extend(self.nonce.to_le_bytes());
        return result;
    }

    pub fn hash(&self) -> [u8; 64] {
        let mut hasher = sha3::Keccak512::new();
        hasher.update(self.format());
        return hasher.finalize().into();
    }
}

#[derive(Clone)]
pub struct Block {
    pub hashable_block: HashableBlock,
    pub hash: [u8; 64],
}

pub fn create_block(
    transactions: Vec<Transaction>,
    previous_hash: [u8; 64],
    previous_index: u64,
    nonce: u64,
) -> Block {
    let index = previous_index + 1;
    let block_data = BlockData {
        id: index,
        transactions: transactions,
        previous_hash: previous_hash,
    };
    let hashable_block = HashableBlock {
        block: block_data,
        nonce: nonce,
    };
    let hash = hashable_block.hash();
    Block {
        hashable_block: hashable_block,
        hash: hash,
    }
}

pub fn mine_block(
    transactions: Vec<Transaction>,
    previous_hash: [u8; 64],
    previous_index: u64,
) -> Block {
    let mut nonce = 0;
    let block_data = BlockData {
        id: previous_index + 1,
        transactions: transactions,
        previous_hash: previous_hash,
    };
    let mut hashable_block = HashableBlock {
        block: block_data,
        nonce: nonce,
    };
    let mut hash = [255; 64];
    // The current minimum difficulty is 2 bytes
    while hash[0..2] != [0; 2] {
        nonce += 1;
        hashable_block.nonce = nonce;
        hash = hashable_block.hash();
    }
    return Block {
        hashable_block: hashable_block,
        hash: hash,
    };
}
