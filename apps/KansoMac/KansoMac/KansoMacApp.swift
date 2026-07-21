//
//  KansoMacApp.swift
//  KansoMac
//
//  Created by Ngonidzashe  Mangudya on 2026/05/29.
//

import SwiftUI
import AppKit
import UniformTypeIdentifiers

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
            CommandGroup(replacing: .newItem) {
                Button("New Note") { store.createNote() }
                    .keyboardShortcut("n", modifiers: .command)
                Button("Quick Open") { store.isCommandPalettePresented = true }
                    .keyboardShortcut("k", modifiers: .command)
            }
            CommandMenu("Editor") {
                Button("Edit Mode") { store.editorMode = .edit }
                    .keyboardShortcut("1", modifiers: [.command, .option])
                Button("Preview Mode") { store.editorMode = .preview }
                    .keyboardShortcut("2", modifiers: [.command, .option])
                Button("Split Mode") { store.editorMode = .split }
                    .keyboardShortcut("3", modifiers: [.command, .option])
            }
            CommandMenu("Library") {
                Button("Import Markdown...") { importMarkdown() }
                    .keyboardShortcut("i", modifiers: [.command, .option])
                Button("Export Current Notebook...") { exportCurrentNotebook() }
                    .keyboardShortcut("e", modifiers: [.command, .option])
            }
        }
    }

    private func importMarkdown() {
        let panel = NSOpenPanel()
        panel.allowsMultipleSelection = true
        panel.canChooseFiles = true
        panel.canChooseDirectories = false
        panel.allowedContentTypes = [
            UTType(filenameExtension: "md"),
            UTType(filenameExtension: "markdown")
        ].compactMap { $0 }

        guard panel.runModal() == .OK else { return }
        let files = panel.urls.compactMap { url -> ImportFileDto? in
            guard let content = try? String(contentsOf: url, encoding: .utf8) else { return nil }
            return ImportFileDto(filename: url.lastPathComponent, content: content)
        }
        _ = store.importMarkdownFiles(files)
    }

    private func exportCurrentNotebook() {
        let panel = NSOpenPanel()
        panel.canChooseFiles = false
        panel.canChooseDirectories = true
        panel.canCreateDirectories = true
        panel.allowsMultipleSelection = false
        panel.prompt = "Export"

        guard panel.runModal() == .OK, let directory = panel.url else { return }
        _ = store.exportCurrentNotebook(to: directory)
    }
}
