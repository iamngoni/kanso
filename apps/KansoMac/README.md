# KansoMac

The native macOS shell — a quiet three-pane SwiftUI app over the shared Rust
engine via the generated UniFFI bindings. It is a SwiftPM package (not part of
the Cargo workspace).

## Build & run (verified working)

```sh
# 1. Build the host FFI static lib + generate Swift bindings (one-time / on FFI change)
cargo build -p kanso-ffi
cargo run -p kanso-ffi --bin uniffi-bindgen -- generate \
  --library target/debug/libkanso_ffi.dylib --language swift \
  --out-dir target/bindings/swift
cp target/bindings/swift/kanso_ffi.swift apps/KansoMac/Sources/KansoMac/
cp target/bindings/swift/kanso_ffiFFI.h  apps/KansoMac/Sources/kanso_ffiFFI/include/

# 2. Build and launch the app
cd apps/KansoMac
swift build      # links against ../../target/debug/libkanso_ffi.a
swift run        # opens the three-pane window on a desktop session
```

`swift build` succeeds and the binary launches: on first run it creates the
engine database (with migrations) at `~/Library/Application Support/Kanso/kanso.db`
through the FFI — the SwiftUI → UniFFI → Rust engine → SQLite path works end to end.

`Package.swift` hardcodes the repo's `target/debug` path for the linker; adjust
if you move the repo.

## Shippable .app / iOS

For a distributable bundle or iOS, build the universal xcframework via
`scripts/build-apple-bindings.sh` and use an Xcode app target (embed
`KansoFFI.xcframework` + add `kanso_ffi.swift` to sources) instead of SwiftPM.

## What it does

- Sidebar lists notebooks (`engine.listNotebooks`)
- Middle pane lists / searches notes (`engine.listNotes` / `engine.searchNotes`)
- Editor edits a note body; every change flows back through
  `engine.updateNoteBody`, which re-indexes FTS and snapshots a revision
- `⌘N` creates a note; the toolbar `+` creates a notebook

The app holds no product truth — it renders the engine's. Sync, sketches, and
MCP attach at this same boundary as they come online.
