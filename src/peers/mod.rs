use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

#[derive(Serialize, Deserialize)]
pub struct Peer {
    pub address: String,
    pub port: u16,
    last_seen: u128,
}

impl Peer {
    pub fn new(address: String, port: u16) -> Self {
        Peer {
            address: address,
            port: port,
            last_seen: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PeerList {
    last_added: u128,
    pub peers: Vec<Peer>,
}

impl PeerList {
    pub fn new() -> Self {
        PeerList {
            last_added: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            peers: Vec::new(),
        }
    }

    pub fn load_peers() -> Result<Self, &'static str> {
        let peer_list_json = std::fs::read_to_string("peers.json");
        if let Ok(peer_list_json) = peer_list_json {
            let peer_list: Result<PeerList, serde_json::Error> =
                serde_json::from_str(&peer_list_json);
            if let Ok(peer_list) = peer_list {
                return Ok(peer_list);
            }
            return Err("Error loading peers");
        }
        return Err("Peers not found");
    }

    pub fn save_peers(&self) {
        let peer_list_string = serde_json::to_string(&self).unwrap();
        std::fs::write("peers.json", peer_list_string).unwrap();
    }

    pub async fn add_peer(&mut self, peer: Peer) {
        // Testing the peer connection
        let stream = TcpStream::connect(format!("{}:{}", peer.address, peer.port)).await;
        let mut stream = match stream {
            Ok(stream) => stream,
            Err(_) => {
                panic!("Error connecting to peer");
            }
        };
        stream.write_all(b"DAN-COIN-PROTOCOL").await.unwrap();
        // Adding the peer to the list
        self.peers.push(peer);
        self.last_added = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        self.save_peers();
    }
}
