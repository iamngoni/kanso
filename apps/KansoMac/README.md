# KansoMac

The native macOS shell — a quiet three-pane SwiftUI app over the shared Rust
engine via the generated UniFFI bindings.

This target is **not** part of the Cargo workspace and is built in Xcode.

## Build the bindings first

```sh
./scripts/build-apple-bindings.sh
```

This produces:

- `target/bindings/swift/kanso_ffi.swift` — the Swift glue (defines `KansoEngine`,
  `NoteDto`, `NotebookDto`, `TagDto`, etc.)
- `target/KansoFFI.xcframework` — static libs + C headers for macOS + iOS

## Wire it into Xcode

1. Create a macOS App target (or SwiftPM executable) from `Sources/KansoMac/`.
2. Add `target/KansoFFI.xcframework` to **Frameworks, Libraries, and Embedded Content**.
3. Add the generated `target/bindings/swift/kanso_ffi.swift` to the target's sources.
4. Build & run.

## Expected editor diagnostics before step 1–3

Opening the `Sources/KansoMac/*.swift` files on their own (e.g. in this repo
without the Xcode target) will show "Cannot find type `KansoEngine`/`NoteDto`"
and "`@main` cannot be used in a module with top-level code". Both are artifacts
of SourceKit analyzing loose files without (a) the generated `kanso_ffi.swift`
and (b) a real app-target module. They resolve once the bindings are generated
and the files are part of the Xcode/SwiftPM target. The Swift here is written
against the exact generated API surface.

## What it does

- Sidebar lists notebooks (`engine.listNotebooks`)
- Middle pane lists / searches notes (`engine.listNotes` / `engine.searchNotes`)
- Editor edits a note body; every change flows back through
  `engine.updateNoteBody`, which re-indexes FTS and snapshots a revision
- `⌘N` creates a note; the toolbar `+` creates a notebook

The app holds no product truth — it renders the engine's. Sync, sketches, and
MCP attach at this same boundary as they come online.
