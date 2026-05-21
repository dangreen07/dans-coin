use transactions::Recipient;
use transactions::create_transaction;
use wallet::create_wallet;

pub mod blockchain;
pub mod transactions;
pub mod wallet;

fn display_key(key: &[u8]) -> String {
    return hex::encode(key);
}

fn main() {
    let wallet = create_wallet();
    let public_key = wallet.public_key;
    let secret_key = wallet.secret_key;

    println!("Public key: {}", display_key(public_key.as_bytes()));
    println!("Secret key: {}", display_key(secret_key.as_bytes()));

    let transaction = create_transaction(
        public_key.to_bytes(),
        10.0,
        0.0,
        vec![Recipient::new(public_key.to_bytes(), 1.0)],
        secret_key,
    );
    let genesis_block = blockchain::mine_block(vec![transaction], [0; 64], 0);
    println!("Genesis block hash: {}", hex::encode(genesis_block.hash));
}
