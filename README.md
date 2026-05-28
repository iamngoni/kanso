# Kanso

Native Markdown notebooks. Simple, portable, local-first, and agent-ready.

This repository holds the shared Rust core. Native apps (SwiftUI for macOS/iOS,
Kotlin for Android) sit on top of `kanso-engine` via UniFFI bindings — they are
not in this workspace.

See the architecture spec (Inkdrop: *Kanso — Architecture & Engine Spec*) for
the full design.

## Workspace

| Crate | Role |
|-------|------|
| `kanso-types`  | Shared domain IDs + sync wire types. Used by both the engine and the cloud server so the protocol can't drift. |
| `kanso-engine` | The product engine: SQLite (canonical), Markdown indexing, FTS5 search, revisions, soft deletes, and the sync outbox. Single source of product truth. |
| `kanso-ink`    | Cross-platform ink core: canonical CBOR sketch format, stroke geometry (smoothing, pressure→width), and headless `tiny-skia` preview/export. No GPU. |
| `kanso-cloud`  | Kanso Cloud sync service (`actix-web`): ordered event replication per user. |

## Stack

Matches the conventions in the sibling backends (heimdall) and workspaces (stanza):
`actix-web` · `sqlx` · `anyhow`/`thiserror` · `uuid` v7 · `chrono` · `log`/`env_logger`,
edition 2024, toolchain 1.93.1.

## Build

```sh
cargo check --workspace
cargo test --workspace
cargo run -p kanso-cloud   # starts the sync service on 127.0.0.1:8787
```

## Notes

- The engine currently uses runtime `sqlx::query` calls. Once the schema is
  frozen we switch the hot paths to the compile-time-checked `query!` macros and
  commit `.sqlx/` offline metadata so iOS/Android cross-builds need no live DB.
- `kanso-cloud` ships with an in-memory event store so the protocol runs
  end-to-end without provisioning Postgres; the Postgres-backed store is the
  production target and slots in behind the same `EventStore` boundary.
