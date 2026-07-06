//
//  KansoMacApp.swift
//  KansoMac
//
//  Created by Ngonidzashe  Mangudya on 2026/05/29.
//

import SwiftUI

// A quiet, native three-pane Markdown notebook over the shared Rust engine
// (via the generated UniFFI bindings in `kanso_ffi.swift`). Requires the
// `kanso_ffiFFI` module — link `KansoFFI.xcframework` into this target.
@main
struct KansoMacApp: App {
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
