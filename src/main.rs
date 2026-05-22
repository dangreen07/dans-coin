// use blockchain::database::BlockChain;
// use transactions::Recipient;
// use transactions::Transaction;
// use wallet::Wallet;

use clap::Parser;
use peers::{Message, Peer, PeerMessage};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

use crate::peers::PeerList;

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

    let (tx, mut rx) = mpsc::channel::<PeerMessage>(100);

    // By default, we start a server and also try our peers
    tokio::spawn(async move {
        println!("Starting server...");
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        println!("Listening on {}", address);

        loop {
            let (mut socket, _) = listener.accept().await.unwrap();
            println!("Connection from {}", socket.peer_addr().unwrap());
            let tx = tx.clone();
            tokio::spawn(async move {
                let mut buffer = [0; 1024];
                let mut data: Vec<u8> = Vec::new();
                loop {
                    let n = match socket.read(&mut buffer).await {
                        // socket closed
                        Ok(0) => break,
                        Ok(n) => n,
                        Err(e) => {
                            eprintln!("failed to read from socket; err = {:?}", e);
                            return;
                        }
                    };
                    data.extend_from_slice(&buffer[..n]);
                }
                // Now the socket is closed
                let message = Message::convert_from_bytes(&data);
                if (message.message_type == 0) & (message.data.len() > 0) {
                    socket.write_all(&message.data).await.unwrap(); // Send the message back
                    let peer = Peer::new(address.ip().to_string(), address.port());
                    let mut peer_list = PeerList::load_peers().unwrap();
                    peer_list.add_peer(peer.clone()).await;
                    let message = PeerMessage::new(peer, message, true);
                    tx.send(message).await.unwrap();
                }
            });
        }
    });

    tokio::spawn(async move {
        loop {
            // The pinging of peers is done here every 30 seconds to ensure they are still alive
            let peer_list = PeerList::load_peers().unwrap();
            for peer in peer_list.peers.iter() {
                let message = "PING".as_bytes().to_vec();
                let message = Message::new(0, message);
                let message = message.convert_to_bytes();
                let stream = TcpStream::connect(format!("{}:{}", peer.address, peer.port)).await;
                let mut stream = match stream {
                    Ok(stream) => stream,
                    Err(_) => {
                        // TODO: Update peer to dead
                        continue;
                    }
                };
                stream.write_all(&message).await.unwrap();
                // Wait for a response
                let mut buffer = [0; 1024];
                let mut data: Vec<u8> = Vec::new();
                loop {
                    let n = match stream.read(&mut buffer).await {
                        // socket closed
                        Ok(0) => break,
                        Ok(n) => n,
                        Err(e) => {
                            eprintln!("failed to read from socket; err = {:?}", e);
                            return;
                        }
                    };
                    data.extend_from_slice(&buffer[..n]);
                }
                if data.len() > 0 {
                    let message = Message::convert_from_bytes(&data);
                    if message.message_type == 0 {
                        // TODO: Update peer to alive
                        println!("Peer {} is alive", peer.address);
                    }
                }
                stream.shutdown().await.unwrap();
            }
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        }
    });

    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C");
    Ok(())
}
