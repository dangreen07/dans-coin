CREATE TABLE IF NOT EXISTS blocks (
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
timestamp INTEGER NOT NULL,
block_id INTEGER NOT NULL,
FOREIGN KEY(block_id) REFERENCES blocks(id)
);

CREATE TABLE IF NOT EXISTS recipients (
transaction_id BLOB,
FOREIGN KEY(transaction_id) REFERENCES transactions(hash_id),
address BLOB NOT NULL,
amount FLOAT NOT NULL
);
