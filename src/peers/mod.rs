use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use tokio::io::AsyncReadExt;
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

    // For when we are checking the status of a peer
    pub async fn add_peer_with_protocol(&mut self, peer: Peer) {
        // Check if the peer is already in the list
        for peer in self.peers.iter() {
            if peer.address == peer.address {
                return;
            }
        }
        // Testing the peer connection
        let stream = TcpStream::connect(format!("{}:{}", peer.address, peer.port)).await;
        let mut stream = match stream {
            Ok(stream) => stream,
            Err(_) => {
                panic!("Error connecting to peer");
            }
        };
        let my_address = std::fs::read_to_string("my_address.json").unwrap();
        let my_address: Result<Peer, serde_json::Error> = serde_json::from_str(&my_address);
        let my_address = match my_address {
            Ok(my_address) => my_address,
            Err(_) => {
                panic!(
                    "Error loading your address, try running the server before connecting to a peer"
                );
            }
        };
        let ping_message = PingMessage::new(my_address.port, 1);
        println!(
            "Sending Ping Message: {}, {}",
            ping_message.listening_port, ping_message.protocol_version
        );
        let ping_message = ping_message.convert_to_bytes();
        let our_message = Message::new(0, ping_message);
        let our_message = our_message.convert_to_bytes();
        stream.write_all(&our_message).await.unwrap();
        stream.write_all(b"\n").await.unwrap(); // Signal end of message
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
            if data.ends_with(b"\n") {
                // Message complete
                data.pop(); // Remove delimiter
                break;
            }
        }
        let message = Message::convert_from_bytes(&data);
        if (message.message_type == 0) & (message.data.len() > 0) {
            // Adding the peer to the list
            self.peers.push(peer);
            self.last_added = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis();
            self.save_peers();
        } else {
            println!("Peer {} is not using the correct protocol", peer.address);
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PingMessage {
    pub listening_port: u16,
    pub protocol_version: u8,
}

impl PingMessage {
    pub fn new(listening_port: u16, protocol_version: u8) -> Self {
        PingMessage {
            listening_port: listening_port,
            protocol_version: protocol_version,
        }
    }

    pub fn convert_to_bytes(&self) -> Vec<u8> {
        let mut message_bytes: Vec<u8> = Vec::new();
        message_bytes.extend_from_slice(&self.listening_port.to_le_bytes());
        message_bytes.push(self.protocol_version); // Single byte, no need for to_le_bytes
        return message_bytes;
    }

    pub fn convert_from_bytes(message_bytes: &[u8]) -> Self {
        let listening_port = u16::from_le_bytes([message_bytes[0], message_bytes[1]]);
        let protocol_version = message_bytes[2];
        PingMessage {
            listening_port,
            protocol_version,
        }
    }
}

// Message types
// 0: Ping message, establishing peer connection

#[derive(Clone)]
pub struct Message {
    pub version: u8,
    pub message_type: u16,
    pub data: Vec<u8>,
}

impl Message {
    pub fn new(message_type: u16, data: Vec<u8>) -> Self {
        Message {
            version: 1,
            message_type: message_type,
            data: data,
        }
    }

    pub fn convert_to_bytes(&self) -> Vec<u8> {
        let mut message_bytes: Vec<u8> = Vec::new();
        let peer_message_type = self.message_type.to_le_bytes();
        let version = self.version.to_le_bytes();
        message_bytes.extend_from_slice(&version);
        message_bytes.extend_from_slice(&peer_message_type);
        message_bytes.extend_from_slice(&self.data);
        return message_bytes;
    }

    pub fn convert_from_bytes(message_bytes: &[u8]) -> Self {
        let version = u8::from_le_bytes([message_bytes[0]]);
        let message_type = u16::from_le_bytes([message_bytes[1], message_bytes[2]]);
        let data = message_bytes[3..].to_vec();
        Message {
            message_type: message_type,
            data: data,
            version: version,
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
