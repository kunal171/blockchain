# Gossip & Peer Discovery — Concepts and Theory

Deep reference for Milestone 0, the networking foundation the blockchain sits on. Read this to understand *why* the gossip layer is built the way it is.

---

## The Problem

A blockchain has no central server. Hundreds of nodes, run by strangers, need to:

1. **Find each other** — a new node knows one or two addresses; it must discover the rest.
2. **Spread information** — a new transaction or block created at one node must reach every other node, fast and reliably, without a broadcast server.
3. **Notice failures** — nodes crash and vanish silently; the network must detect and route around them.

Doing this with direct connections (everyone talks to everyone) is `O(n²)` connections and collapses at scale. **Gossip protocols** solve it with `O(n)` work per node and `O(log n)` rounds to reach everyone.

---

## The Core Idea: Epidemic Spread

Gossip copies how rumors (or diseases) spread. Each node, periodically:

1. Picks a small random subset of peers (say `k = 3`).
2. Sends them whatever new information it has.
3. Those peers do the same next round.

Information spreads exponentially: round 1 one node knows, round 2 up to `k` know, round 3 up to `k²`, and so on. A network of 1000 nodes is fully infected in ~10 rounds. No node is a bottleneck, and no single failure stops the spread — there are always many paths.

```
round 0:  A
round 1:  A → B, C, D
round 2:  B → E, F, G   C → H, I, J   D → K, L, M
round 3:  ... exponential ...
```

**Why random peers?** Fixed neighbors create predictable paths that a single failure or partition can cut. Random selection means many independent paths — the property that makes gossip robust.

---

## Peer Discovery

A node boots knowing only a **seed** (one or a few well-known addresses). Discovery bootstraps the rest:

1. New node connects to the seed and sends a `Join` (its own id/address).
2. The seed replies with its current **peer list**.
3. The new node adds those peers and gossips its own arrival, so the network learns about it.
4. Over time, every node accumulates a view of the membership.

This is the same shape as Bitcoin's DNS seeds + `addr` messages and Ethereum's discovery protocol — just simplified.

---

## Membership and Node State

Each node keeps a table of the peers it knows, each tagged with a state:

```
Alive    — recently heard from; healthy
Suspect  — missed a health check; maybe dead, maybe just slow
Dead     — confirmed unreachable; will be dropped
```

A peer moves `Alive → Suspect → Dead` as health checks fail, and snaps back to `Alive` the moment it responds again. Tracking *suspect* (not jumping straight to dead) avoids evicting a peer that was merely slow for one round.

---

## Failure Detection — the SWIM Pattern

SWIM (Scalable Weakly-consistent Infection-style Membership) is the standard gossip failure detector. Each round, a node:

1. **Direct ping** — picks a random peer and sends `Ping`, expecting an `Ack`.
2. **Indirect ping** — if no ack, it asks `k` *other* peers to ping the target on its behalf (`PingReq`). This distinguishes "the target is dead" from "*my* link to the target is bad."
3. **Suspect → Dead** — still no ack (direct or indirect) → mark `Suspect`, then `Dead` after a timeout, and gossip that fact so others update too.

```
A --Ping--> X        (no Ack)
A --PingReq(X)--> B, C
B --Ping--> X --Ack--> B --Ack--> A     (X is alive; A's link was bad)
   ...or no one hears back → X is Dead
```

Indirect probing is what makes SWIM accurate: it avoids false positives from a single flaky connection.

---

## Anti-Entropy (Reconciliation)

Pure push-gossip can occasionally miss a node (bad luck in random selection). **Anti-entropy** is a periodic background sync where two nodes compare their full membership/state and reconcile any differences. Push-gossip gets information *most* of the way fast; anti-entropy guarantees eventual consistency by mopping up the stragglers.

---

## Push vs Pull vs Push-Pull

- **Push** — "here's what I know." Fast early, wasteful late (everyone already knows).
- **Pull** — "tell me what's new." Efficient late in a round.
- **Push-Pull** — do both in one exchange. Best of both; what most real systems use.

For this project, push dissemination + periodic anti-entropy is enough to demonstrate the ideas.

---

## Message Types (what we'll model)

| Message | Purpose |
|---------|---------|
| `Join` | new node announces itself to a seed |
| `PeerList` | reply carrying known peers |
| `Gossip(payload)` | a piece of data to disseminate (later: a tx or block) |
| `Ping` / `Ack` | direct health check |
| `PingReq` | ask a peer to probe a third node (indirect check) |

These get serialized as length-prefixed JSON over TCP — the same wire pattern as the Phase 2A distributed queue.

---

## How This Feeds the Blockchain

Once the gossip layer works, the chain plugs in with almost no new networking:

- A new **transaction** → wrapped in `Gossip(tx)` → spreads to every node's mempool.
- A newly mined **block** → wrapped in `Gossip(block)` → every node validates and, if the chain is longer/valid, adopts it.
- **Failure detection** keeps the peer set healthy so propagation stays fast.

The gossip layer is deliberately agnostic about *what* it spreads — that's why it's a reusable foundation.

---

## Key Design Trade-offs (interview-worthy)

**Q: Why gossip instead of a central broadcast server?**
No single point of failure, no bottleneck, and it scales to thousands of nodes with `O(log n)` propagation. A central server can't be trusted in a decentralized system anyway.

**Q: How fast does gossip converge?**
About `O(log n)` rounds to reach all `n` nodes, because each round multiplies the number of informed nodes by roughly the fanout `k`.

**Q: Why does SWIM use indirect pings?**
To tell "the target is dead" apart from "my own link to it is flaky," which slashes false-positive failure reports.

**Q: What's the fanout trade-off?**
Higher `k` → faster convergence but more bandwidth and redundant messages. Lower `k` → cheaper but slower and slightly less robust. Real systems tune `k` (often 3–5).

**Q: Push vs pull?**
Push spreads fast when few know; pull is efficient when most already know. Push-pull combines them; anti-entropy guarantees nobody is permanently missed.
