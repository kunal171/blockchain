//! # blockchain
//!
//! A mini blockchain in Rust, built on a gossip networking foundation.
//!
//! The project is layered: a peer-to-peer **gossip** network (Milestone 0)
//! carries messages between untrusting nodes, and the **chain** — blocks,
//! transactions, Merkle trees, Proof of Work, and validation — is built on top
//! of it. Nodes mine blocks and gossip them; peers validate and adopt the
//! longest valid chain.
//!
//! ## Module map (grows per milestone)
//!
//! | Module | Milestone | Responsibility |
//! |--------|-----------|----------------|
//! | `gossip` | M0 | peer discovery, message dissemination, failure detection |
//! | `transaction` | M1 | the `Transaction` type |
//! | `block` | M1 | `Block`, hashing, Merkle root |
//! | `pow` | M2 | Proof of Work mining |
//! | `chain` | M3 | the `Blockchain`, validation, fork resolution |
//!
//! See `GOSSIP_CONCEPTS.md` for the networking theory and `README.md` for the
//! milestone plan.

// Modules are declared here as each milestone is built, e.g.:
pub mod gossip;
