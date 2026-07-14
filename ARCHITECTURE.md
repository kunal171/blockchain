# Architecture

A living design document. It tracks **what exists today** and **what it will become** — updated as each milestone lands.

> **Status:** Milestone 0 (gossip networking) — Step 2 of 5 complete.
> Legend: ✅ built · 🚧 in progress · ⬜ planned

---

## 1. The Big Picture

The system is **two layers**. The lower layer moves bytes between untrusting nodes; the upper layer decides what those bytes *mean*.

```
┌──────────────────────────────────────────────────────────┐
│  CHAIN LAYER  (M1–M3)                              ⬜     │
│  transactions · blocks · Merkle root · PoW ·             │
│  validation · longest-chain fork resolution              │
└─────────────────────────┬────────────────────────────────┘
                          │  publish(tx) / publish(block)
                          │  on_receive(tx) / on_receive(block)
┌─────────────────────────▼────────────────────────────────┐
│  GOSSIP LAYER  (M0)                               🚧     │
│  peer discovery · epidemic dissemination ·               │
│  SWIM failure detection                                  │
│  Tokio + TCP, length-prefixed JSON                       │
└──────────────────────────────────────────────────────────┘
```

**Why this split:** the gossip layer is deliberately *agnostic* about what it spreads. It moves opaque payloads. That makes it reusable and independently testable — and it mirrors how real chains work (libp2p under Substrate, devp2p under Ethereum).

Each process runs **one node** that owns both layers:

```
   Node A  ⇄  Node B  ⇄  Node C  ⇄  Node D
     │          │          │          │
   chain      chain      chain      chain      ← same code, independent state
   gossip     gossip     gossip     gossip
```

There is no leader and no server. This is the key departure from Raft (Phase 2C): agreement comes from hashing + Proof of Work + the longest-chain rule, not from an elected authority.

---

## 2. Module Map

| Module | Milestone | Status | Responsibility |
|--------|-----------|--------|----------------|
| `gossip::peer` | M0 | ✅ | `PeerId`, `PeerState`, `Peer` — the membership model |
| `gossip::message` | M0 | ✅ | `Message` enum — the entire wire protocol |
| `gossip::transport` | M0 | ✅ | length-prefixed JSON codec over TCP |
| `gossip::node` | M0 | 🚧 | `GossipNode`: state, listener, message dispatch |
| *discovery logic* | M0 | ⬜ | `join_seed`, handling `Join` / `PeerList` |
| *dissemination* | M0 | ⬜ | push `Gossip` to `k` random peers; dedup |
| *failure detection* | M0 | ⬜ | SWIM `Ping` / `Ack` / `PingRequest` rounds |
| `transaction` | M1 | ⬜ | the `Transaction` type |
| `block` | M1 | ⬜ | `Block`, SHA-256 header hash, Merkle root |
| `pow` | M2 | ⬜ | mining loop, difficulty |
| `chain` | M3 | ⬜ | `Blockchain`, validation, fork resolution |

---

## 3. Data Model (built ✅)

### Peer — `gossip/peer.rs`

```rust
type PeerId = u64;

enum PeerState { Alive, Suspect, Dead }

struct Peer {
    id: PeerId,
    addr: String,      // "127.0.0.1:9001"
    state: PeerState,
}
```

`PeerId` is kept **separate from `addr`** so a node keeps its identity even if its address changes.

`PeerState` is the SWIM lifecycle. `Suspect` is the important middle state — it prevents one missed ping from evicting a peer that was merely slow:

```
   Alive ──miss probe──> Suspect ──timeout──> Dead
     ▲                      │
     └──────responds────────┘
```

### Message — `gossip/message.rs`

The **entire protocol** is one enum. Every network interaction is a variant.

```rust
enum Message {
    Join        { id, addr },          // discovery: announce self to a seed
    PeerList    { peers },             // discovery: seed's reply
    Gossip      { payload },           // dissemination: opaque data to spread
    Ping        { from },              // failure detection: direct probe
    Ack         { from },              // failure detection: probe reply
    PingRequest { from, target },      // failure detection: indirect probe (SWIM)
}
```

All variants derive `Serialize`/`Deserialize` — unlike Raft's messages, these leave the process and must survive the wire.

---

## 4. Wire Protocol (built ✅)

TCP is a **byte stream with no message boundaries**. Framing solves that:

```
┌────────────────┬──────────────────────────────┐
│  4-byte length │   JSON-encoded Message       │
│  (big-endian)  │   (`length` bytes)           │
└────────────────┴──────────────────────────────┘
```

**Read path:** read exactly 4 bytes → interpret as `u32` length → read exactly that many bytes → `serde_json::from_slice` → `Message`.

Without the prefix, a reader can't tell where one JSON document ends and the next begins. This is the same framing as the Phase 2A distributed queue.

`transport.rs` exposes three functions:

| Function | Purpose |
|----------|---------|
| `send_message(addr, msg)` | connect to `addr`, send one message, done |
| `write_message(stream, msg)` | frame + write onto an existing stream |
| `read_message(stream)` | read exactly one framed message |

**Connection model:** one message per connection (connect → send → close). Simple, and adequate for gossip's low message rate. A production system would pool connections.

---

## 5. Node & Concurrency Model (🚧)

```rust
struct GossipNode {
    id: PeerId,
    addr: String,
    peers: Arc<Mutex<HashMap<PeerId, Peer>>>,   // the membership table
}
```

### Why `Arc<Mutex<...>>`

The membership table is touched from several places at once:

```
        ┌──────────────┐
        │ accept loop  │──spawn──> task ──> handle_message ──┐
        └──────────────┘                                      │
        ┌──────────────┐                                      ├──> peers
        │ gossip round │ (M0 step 4, background task) ────────┤     (shared)
        └──────────────┘                                      │
        ┌──────────────┐                                      │
        │ ping round   │ (M0 step 5, background task) ────────┘
        └──────────────┘
```

`Arc` shares ownership across tasks; `Mutex` serializes access. Same shape as `Arc<Mutex<Broker>>` in the distributed queue.

**`tokio::sync::Mutex`, not `std::sync::Mutex`** — the lock is held across `.await` points. A std guard isn't `Send` across an await and would deadlock the runtime.

### The accept loop

```
listen()
  └─ TcpListener::bind(addr)
  └─ loop {
        accept()  ──>  tokio::spawn(handle one message)  ──> back to accept immediately
     }
```

Spawning per connection means **one slow or dead peer can't stall the node**. `GossipNode` derives `Clone` so each task gets a cheap handle (it's just an `Arc` clone).

### Message dispatch

`handle_message` is a single `match` over `Message`. Every remaining M0 step is just **filling in one more arm** — the skeleton doesn't change.

---

## 6. Planned Flows (⬜)

### Discovery (M0 · Step 3)

```
new node                          seed
   │  ── Join { id, addr } ───────>│
   │                                │ adds peer to table
   │  <───── PeerList { peers } ────│
   │ populates its own table        │
```

Bootstraps membership from a single known address. Same shape as Bitcoin's DNS seeds + `addr` exchange.

### Dissemination (M0 · Step 4)

Epidemic push: each node forwards a new payload to `k` random peers, who do the same. Reaches all `n` nodes in ~`O(log n)` rounds.

```
round 0:  A
round 1:  A → B, C, D
round 2:  B → E, F, G    C → H, I, J    D → K, L, M
```

Needs a **seen-set** for dedup, or a payload would circulate forever.

### Failure detection (M0 · Step 5)

SWIM. Direct probe, then *indirect* probe through `k` other peers — which distinguishes "the target is dead" from "*my link* to the target is bad."

```
A --Ping--> X            (no Ack)
A --PingRequest(X)--> B, C
   B --Ping--> X --Ack--> B --Ack--> A     → X alive, A's link was bad
   ...silence from everyone               → X: Alive → Suspect → Dead
```

### Chain over gossip (M4)

The payoff of the layering — the chain publishes onto gossip and reacts to what arrives:

```
mine a block ──> Gossip { payload: block } ──> spreads to all peers
                                                     │
peer receives ──> validate ──> longer valid chain? ──> adopt
```

---

## 7. Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| Gossip layer is payload-agnostic | Reusable and testable on its own; the chain plugs in without touching networking |
| Length-prefixed JSON | Human-readable while learning; framing is mandatory over TCP regardless of encoding |
| One message per connection | Simple; gossip's message rate doesn't justify pooling |
| `PeerId` separate from address | Identity survives an address change |
| `Suspect` state before `Dead` | Avoids evicting merely-slow peers on a single missed probe |
| Random peer selection | Many independent propagation paths → robust to partitions and failures |
| `tokio::spawn` per connection | A dead peer can't block the accept loop |

---

## 8. Deliberate Simplifications

This is a learning implementation. Compared to a real chain it omits:

- **cryptographic signatures** — transactions aren't signed; no wallets or key management
- **UTXO / account model** — no balances, just opaque payloads for now
- **connection pooling / NAT traversal / encryption**
- **persistence** — chain and membership live in memory
- **difficulty retargeting over time** — difficulty is a fixed parameter

---

*Updated as each milestone lands. Theory lives in [GOSSIP_CONCEPTS.md](GOSSIP_CONCEPTS.md); the milestone plan is in [README.md](README.md).*
