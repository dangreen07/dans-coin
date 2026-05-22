use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

#[derive(Serialize, Deserialize, Clone)]
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
        return Ok(Self::new());
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

// Message types
// 0: Test Message

#[derive(Clone)]
pub struct Message {
    pub message_type: u16,
    pub data: Vec<u8>,
}

impl Message {
    pub fn new(message_type: u16, data: Vec<u8>) -> Self {
        Message {
            message_type: message_type,
            data: data,
        }
    }

    pub fn convert_to_bytes(&self) -> Vec<u8> {
        let mut message_bytes: Vec<u8> = Vec::new();
        let peer_message_type = self.message_type.to_le_bytes();
        message_bytes.extend_from_slice(&peer_message_type);
        message_bytes.extend_from_slice(&self.data);
        return message_bytes;
    }

    pub fn convert_from_bytes(message_bytes: &[u8]) -> Self {
        let message_type = message_bytes[0..2].to_vec();
        let message_type = u16::from_le_bytes(message_type.try_into().unwrap());
        let data = message_bytes[1..].to_vec();
        Message {
            message_type: message_type,
            data: data,
        }
    }
}

#[derive(Clone)]
pub struct PeerMessage {
    pub recieved: bool,
    pub peer: Peer,
    pub message: Message,
}

impl PeerMessage {
    pub fn new(peer: Peer, message: Message, recieved: bool) -> Self {
        PeerMessage {
            recieved: recieved,
            peer: peer,
            message: message,
        }
    }
}
