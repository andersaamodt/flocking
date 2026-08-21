# Flocking project standards

- [SPEC.md](../SPEC.md) is the semantic arche for Flocking v1.
- [.github/INCLUSION_LEDGER.yaml](INCLUSION_LEDGER.yaml) is the dependency and capability-selection arche.
- Rust is the explicit non-shell implementation boundary approved by the project owner on 2026-08-20.
- `flocking-core` owns pure semantics and must not access networks, storage, clocks, signers, or UI state.
- `flocking-nostr` owns Nostr translation and must not fetch relays, retain keys, or reinterpret core results.
- Portable configuration and normative vectors are versioned JSON, while ordinary application storage remains outside this repository and library.
- Runnable repository verification enters through `.tests/run`, with focused Rust tests beside their owning crates.
- Cargo build output and transient test state remain outside the repository during canonical verification.
- The current Rust boundary, JSON formats, Nostr adapter, and AGPL license are approved project standards rather than exceptions.
- No local exceptions or pending standards decisions are currently recorded.
