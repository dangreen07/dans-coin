use rusqlite::Connection;

use crate::blockchain::Block;

pub fn initialize_database() {
    let connection = Connection::open("blockchain.sqlite").unwrap();

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

    connection.close().unwrap();
}

pub fn add_block(block: &Block) -> Result<&Block, rusqlite::Error> {
    let connection = Connection::open("blockchain.sqlite").unwrap();
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
        let result = transaction.add_to_database(&connection, block);
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
