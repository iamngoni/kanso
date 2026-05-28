import SwiftUI

// The Kanso macOS shell. A quiet, native three-pane Markdown notebook over the
// shared Rust engine (via the generated UniFFI bindings in `kanso_ffi.swift`).
//
// This target is built in Xcode against `KansoFFI.xcframework` + `kanso_ffi.swift`,
// both produced by `scripts/build-apple-bindings.sh`. It is intentionally not part
// of the Cargo workspace.
@main
struct KansoApp: App {
    @StateObject private var store = KansoStore()

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(store)
                .frame(minWidth: 1100, minHeight: 720)
        }
        .windowStyle(.hiddenTitleBar)
        .commands {
            CommandGroup(after: .newItem) {
                Button("New Note") { store.createNote() }
                    .keyboardShortcut("n", modifiers: .command)
            }
        }
    }
}
