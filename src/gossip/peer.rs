//! This module defines the Peer struct, which represents a peer in the gossip network.
use serde::{Serialize, Deserialize};
/// A uniquue ID for a node in the gossip network.
/// 
pub type PeerId = u64;

/// Health state of a peer in the gossip network.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeerState {
    ALive, //Recently heard from
    Suspect, //missed a health Check - Maybe dead, Maybe slow
    Dead, // confirmed unreachable will be dropped 
}

/// A peer this node knows about in the gossip network. 
/// It has a unique ID, an address, and a state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Peer {
    pub id: PeerId,
    pub addr: String, //IP address and port of the peer e.g."127.0.0.1:8080"
    pub state: PeerState,
}

