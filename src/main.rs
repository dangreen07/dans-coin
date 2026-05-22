// use blockchain::database::BlockChain;
// use transactions::Recipient;
// use transactions::Transaction;
// use wallet::Wallet;

use clap::Parser;

pub mod blockchain;
pub mod peers;
pub mod transactions;
pub mod wallet;

// fn display_key(key: &[u8]) -> String {
//     return hex::encode(key);
// }

// fn main() {
//     let wallet = match Wallet::load_wallet() {
//         Ok(wallet) => wallet,
//         Err(_) => {
//             let wallet = Wallet::new();
//             wallet.save_wallet();
//             wallet
//         }
//     };
//     let public_key = wallet.public_key;
//     let secret_key = wallet.secret_key;

//     println!("Public key: {}", display_key(public_key.as_bytes()));
//     println!("Secret key: {}", display_key(secret_key.as_bytes()));

//     let transaction = Transaction::create_transaction(
//         public_key.to_bytes(),
//         10.0,
//         0.0,
//         vec![Recipient::new(public_key.to_bytes(), 1.0)],
//         secret_key,
//     );
//     let mut genesis_block = blockchain::Block::new(vec![transaction], [0; 64], 0, 0);
//     genesis_block.mine();

//     let blockchain = BlockChain::new();
//     let block_id = genesis_block.hashable_block.block.id;
//     let genesis_block = match blockchain.add_block(genesis_block) {
//         Ok(genesis_block) => genesis_block,
//         Err(_) => match blockchain.get_block(block_id) {
//             Ok(genesis_block) => genesis_block,
//             Err(_) => {
//                 println!("Error adding genesis block");
//                 return;
//             }
//         },
//     };
//     println!("Genesis block hash: {}", hex::encode(genesis_block.hash));
//     println!("Blockchain length: {}", blockchain.length());
// }

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, name = "peer")]
    add_peer: Option<String>,

    #[arg(short, long)]
    list_peers: bool,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    if cli.list_peers {
        let peer_list = peers::PeerList::load_peers();
        if let Ok(peer_list) = peer_list {
            if peer_list.peers.len() == 0 {
                println!("There are no peers!");
            } else {
                println!("Peers:");
                for peer in peer_list.peers.iter() {
                    println!("{}:{}", peer.address, peer.port);
                }
            }
        } else {
            println!("There are no peers!");
            peers::PeerList::new().save_peers();
        }
        return Ok(());
    }
    if let Some(peer) = cli.add_peer.as_deref() {
        let mut peer_list = peers::PeerList::load_peers().unwrap();
        let peer = peer.split(":").collect::<Vec<&str>>();
        let address = peer[0].to_string();
        let port = peer[1].parse::<u16>().unwrap();
        peer_list.add_peer(peers::Peer::new(address, port)).await;
        peer_list.save_peers();
        return Ok(());
    }

    // By default, we start a server and also try our peers
    tokio::spawn(async move {
        println!("Starting server...");
    })
    .await
    .unwrap();
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C");
    Ok(())
}
