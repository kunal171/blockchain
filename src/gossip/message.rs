//! This module defines the Message struct, which represents a message in the gossip network.
//! 

use crate::gossip::peer::{Peer, PeerId};

/// Every message exchanged over the gossip network.
pub enum Message {
    Join { id: PeerId, addr: String }, // A new peer is joining the network

    PeerList { peers: Vec<Peer> }, // A list of known peers

    Gossip {Payload: String}, // A gossip message with a payload

    Ping { id: PeerId }, // A ping message to check if a peer is alive

    Ack { id: PeerId }, // An acknowledgment message in response to a ping

    PingRequest { id: PeerId }, // A request to ping a peer
}