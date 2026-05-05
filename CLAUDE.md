# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project overview

Rust service implementing a simple key-value API with pluggable storage backends:

- **HTTP server**: Axum-based API server.
- **Storage backends**:
  - `memory`: in-memory `HashMap`.
  - `rocksdb`: local RocksDB persistence.
  - `multiple_node`: distributed mode that uses **OpenRaft** with RocksDB-backed log + state machine.

The binary selects the backend via CLI flags; in distributed mode it also initializes a Raft node.

## Common commands

### Build

```bash
cargo build
cargo build --release
```

### Run

```bash
# Default: memory backend on port 9901
cargo run

# Explicit port
cargo run -- --port 9901

# RocksDB backend
cargo run -- --type rocksdb --storage-path /tmp/kv-rocks

# Multiple-node (OpenRaft) backend
# member list format: <node_id>@<ip>:<port>,<node_id>@<ip>:<port>
# Example with 3 nodes:
#  - node 1: 1@127.0.0.1:9901
#  - node 2: 2@127.0.0.1:9902
#  - node 3: 3@127.0.0.1:9903
cargo run -- \
  --type multiple_node \
  --node-id 1 \
  --port 9901 \
  --storage-path /tmp/kv-node-1 \
  --member-list "1@127.0.0.1:9901,2@127.0.0.1:9902,3@127.0.0.1:9903"


cargo run -- \
  --type multiple_node \
  --node-id 2 \
  --port 9902 \
  --storage-path /tmp/kv-node-2 \
  --member-list "1@127.0.0.1:9901,2@127.0.0.1:9902,3@127.0.0.1:9903"



cargo run -- \
  --type multiple_node \
  --node-id 3 \
  --port 9903 \
  --storage-path /tmp/kv-node-3 \
  --member-list "1@127.0.0.1:9901,2@127.0.0.1:9902,3@127.0.0.1:9903"    
```

### Tests

```bash
cargo test

# Run a single test by name
cargo test <test_name_substring>

# Run tests in one module/file (via substring)
cargo test storage::
```

### Formatting / lint

```bash
cargo fmt
cargo clippy --all-targets --all-features
```

## HTTP API

Axum routes are built in `src/axum/mod.rs` and `src/axum/api.rs`.

- `GET /health`
- `GET /api/v1/set?key=...&value=...`
- `GET /api/v1/get/{key}`
- `GET /api/v1/del/{key}`

Distributed-related endpoints are currently stubbed under `GET /raft/...` (see `src/axum/raft.rs`).

## Code structure (big picture)

### Entry points / wiring

- `src/main.rs`: initializes tracing, parses CLI (`CliArgs`), builds `AppConfig`, selects storage backend, then starts the Axum server.
- `src/utils/config.rs`: CLI parsing + validation; defines `StorageType` and `AppConfig`.
- `src/storage/mod.rs`: `StorageService` trait, one-time global storage initialization (`init_storage`) and access (`get_storage`).

### Storage backends

- `src/storage/memory/mod.rs`: in-memory backend.
- `src/storage/rocksdb/mod.rs`: RocksDB backend.
- `src/storage/multiple_node/mod.rs`: `MultipleNodeStorage` adapter that delegates reads/writes to the OpenRaft manager.

Shared stored value shape:
- `src/storage/entity.rs`: `DataEntity { key, value, update_time }` serialized as JSON bytes.

### OpenRaft (distributed mode)

The OpenRaft integration lives under:
- `src/storage/multiple_node/openraft_setting/`

Key components:

- `type_config.rs`: OpenRaft `TypeConfig` declaration (request/response node types).
- `mod.rs`: request/response enums (`RaftInnerRequest`, `RaftInnerResponse`) used as the Raft application payload.
- `manager.rs`: `OpenRaftManagerService` creates:
  - RocksDB-backed **log store** and **state machine** stores,
  - a `Raft<TypeConfig>` instance,
  - cluster initialization via `initialize(nodes)` if not already initialized.
  It exposes:
  - `write_value()`: `client_write()`; on forwarding-to-leader errors it proxies the write to the leader.
  - `read_value()`: local (potentially stale) reads from the state machine store.
  - `linearizable_read()`: stronger reads via OpenRaft read linearizer (currently unused).

Storage implementation details:
- `store/log_store.rs`: RocksDB-backed Raft log storage (`RaftLogStorage` + `RaftLogReader`).
- `store/state_machine.rs`: RocksDB-backed Raft state machine (`RaftStateMachine` + snapshot builder).

Networking:
- `network/mod.rs`: `RaftNetworkFactory` creating per-target clients.
- `network/connection.rs`: `RaftNetworkV2` implementation using `reqwest` POSTs to the server’s raft endpoints.
- `network/proxy.rs`: simple forward-to-leader proxy that replays a KV `/api/v1/set` request against the leader.

## Notes for making changes

- The runtime storage backend is a global `OnceCell`; tests or binaries that need to swap storage backends must run in separate processes.
- In `multiple_node` mode, `--member-list` parsing expects entries like `1@127.0.0.1:9901` (validated in `CliArgs::validate`).
