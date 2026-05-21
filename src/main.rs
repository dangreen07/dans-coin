use blockchain::database::BlockChain;
use transactions::Recipient;
use transactions::Transaction;
use wallet::Wallet;

pub mod blockchain;
pub mod transactions;
pub mod wallet;

fn display_key(key: &[u8]) -> String {
    return hex::encode(key);
}

fn main() {
    let wallet = match Wallet::load_wallet() {
        Ok(wallet) => wallet,
        Err(_) => {
            let wallet = Wallet::new();
            wallet.save_wallet();
            wallet
        }
    };
    let public_key = wallet.public_key;
    let secret_key = wallet.secret_key;

    println!("Public key: {}", display_key(public_key.as_bytes()));
    println!("Secret key: {}", display_key(secret_key.as_bytes()));

    let transaction = Transaction::create_transaction(
        public_key.to_bytes(),
        10.0,
        0.0,
        vec![Recipient::new(public_key.to_bytes(), 1.0)],
        secret_key,
    );
    let mut genesis_block = blockchain::Block::new(vec![transaction], [0; 64], 0, 0);
    genesis_block.mine();

    let blockchain = BlockChain::new();
    let block_id = genesis_block.hashable_block.block.id;
    let genesis_block = match blockchain.add_block(genesis_block) {
        Ok(genesis_block) => genesis_block,
        Err(_) => match blockchain.get_block(block_id) {
            Ok(genesis_block) => genesis_block,
            Err(_) => {
                println!("Error adding genesis block");
                return;
            }
        },
    };
    println!("Genesis block hash: {}", hex::encode(genesis_block.hash));
    println!("Blockchain length: {}", blockchain.length());
}
