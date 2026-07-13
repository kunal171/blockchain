use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use crate::gossip::message::Message;
use crate::gossip::peer::{Peer, PeerId};
use crate::gossip::transport::read_message;

/// One node in the gossip network.
#[derive(Clone)]
pub struct GossipNode {
    pub id : PeerId,
    pub addr: String,
    /// Membership table, shared between the accept loop and background tasks.
    pub peers: Arc<Mutex<HashMap<PeerId, Peer>>>,
}

impl GossipNode{
    pub fn new(id: PeerId, addr: impl Into<String>) -> Self {
        GossipNode {
            id,
            addr: addr.into(),
            peers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Bind the listener and serve inbound messages forever.
    pub async fn listen(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind(&self.addr)await?;
        println!("[node {}] listening on {}", self.id, self.addr);

        loop {
            let (mut stream, _) = listener.accept().await?;
            let node = self.clone(); //cheap - just clone the arc
            tokio::spawn(async move {
                if let Ok(msg) = read_message(&mut stream).await {
                    node.handle_message(msg).await;
                }
            });
        }
    }

    /// Dispatch an inbound message. Arms get filled in over the next steps.
    async fn handle_message(&self, msg: Message) {
        match msg {
            Message::Join { id, addr } => { /* Step 3 — discovery */ }
            Message::PeerList { peers } => { /* Step 3 — discovery */ }
            Message::Gossip { payload } => { /* Step 4 — dissemination */ }
            Message::Ping { from } => { /* Step 5 — failure detection */ }
            Message::Ack { from } => { /* Step 5 */ }
            Message::PingRequest { from, target } => { /* Step 5 */ }
        }
    }
}