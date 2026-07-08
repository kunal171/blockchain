# Blockchain

A mini blockchain in Rust, built on a peer-to-peer gossip networking foundation.

Raft (Phase 2C) solved consensus among *trusting* nodes. A blockchain solves it among *untrusting* nodes: anyone can propose a block, so agreement comes from cryptographic hashing, Proof of Work, and a longest-valid-chain rule rather than a single elected leader. This project builds that from scratch, on top of a gossip layer that propagates transactions and blocks across the network.

For the gossip/peer-discovery theory (Milestone 0), see **[GOSSIP_CONCEPTS.md](GOSSIP_CONCEPTS.md)**.

---

## Architecture

```
        ┌───────────────────────────────────────────┐
        │   Chain  (M1–M3)                           │
        │   blocks · transactions · Merkle root ·    │
        │   Proof of Work · validation · fork rule   │
        └───────────────────────┬───────────────────┘
                                │ publish tx / block
        ┌───────────────────────▼───────────────────┐
        │   Gossip  (M0) — the networking foundation │
        │   peer discovery · dissemination ·         │
        │   failure detection   (Tokio + TCP)        │
        └────────────────────────────────────────────┘

   Node A ⇄ Node B ⇄ Node C ⇄ Node D     (each runs chain + gossip)
```

A new block or transaction is published onto the gossip layer at one node and
spreads epidemically to the whole network; each node validates it and, for
blocks, adopts the longest valid chain.

---

## Milestones

**Milestone 0 — Gossip networking foundation** *(current)*
Peer membership, discovery via a seed node, epidemic message dissemination
(push to `k` random peers), and SWIM-style failure detection. Tokio + TCP.
Deliverable: start N nodes, publish a message on one, watch it reach all; kill a
node, watch it get detected.
Module: `gossip`

**Milestone 1 — Block & transaction model**
`Transaction`, `Block` (index, timestamp, prev_hash, nonce, transactions,
Merkle root), SHA-256 hashing, and a Merkle tree over the transactions.
Deliverable: build a block, compute its hash and Merkle root, detect tampering.
Modules: `transaction`, `block`

**Milestone 2 — Proof of Work**
The mining loop: find a nonce so `hash(block)` has N leading zeros; difficulty.
Deliverable: mine a block at a target difficulty and time it.
Module: `pow`

**Milestone 3 — The chain**
`Blockchain` struct, genesis block, append-with-validation, full-chain
validation, and the longest-chain rule for resolving forks.
Deliverable: validate a chain; reject a tampered or forked one.
Module: `chain`

**Milestone 4 — Chain over gossip**
Wire M1–M3 onto M0: a node mines a block and gossips it; peers validate and
adopt it; transactions propagate the same way.

**Milestone 5 — Integration tests**
Multi-node scenarios: all nodes converge on the same chain; a fork resolves to
the longest valid chain; an invalid block from a peer is rejected.

---

## Running

```bash
cargo build
cargo test
cargo run
```

---

## Dependencies

- `tokio` — async runtime + TCP for the gossip layer
- `serde` / `serde_json` — wire format for gossip messages
- `sha2` / `hex` — block hashing (Milestone 1+)

---

## Status

Milestone 0 in progress. This is a learning project — see
[GOSSIP_CONCEPTS.md](GOSSIP_CONCEPTS.md) for the theory behind the network layer.
