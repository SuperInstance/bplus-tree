# B+ Tree

A **B+ tree** is a self-balancing tree where all data lives in leaf nodes and internal nodes contain only keys for routing — the standard index structure in databases.

## Why It Matters

B+ trees are optimized for disk I/O: high fanout means shallow trees (3-4 levels for billions of records), sequential leaf access enables range scans, and insertions preserve locality. Every major database (Postgres, MySQL, SQLite) uses B+ trees as their primary index.

## How It Works

Internal nodes have n keys and n+1 children. Leaf nodes contain key-value pairs and are linked in a doubly-linked list for efficient range queries. Splits and merges maintain balance. This implementation supports configurable page size and fanout.

## Usage

```toml
[dependencies]
bplus-tree = "0.1.0"
```

```rust
use bplus_tree;

// See examples/ directory for detailed usage
```

## API

- `BPlusTree` (lib.rs)

## Architecture

This crate is part of the **[SuperInstance](https://github.com/SuperInstance)** ecosystem — a conservation-law-based framework for fleet coordination, ternary computation, and distributed agent systems.

### Related Crates

- [`superinstance-core`](https://github.com/SuperInstance/superinstance-core) — Core conservation law (γ + η = C)
- [`superinstance-harness`](https://github.com/SuperInstance/superinstance-harness) — Build harness and self-improving loop
- [`fleet-coordinator`](https://github.com/SuperInstance/fleet-coordinator) — Fleet-level coordination

## References

- [SuperInstance Architecture](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md)
- [Conservation Law Paper](https://github.com/SuperInstance/SuperInstance/blob/main/docs/conservation-law.md)

## License

MIT
