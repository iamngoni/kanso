import Foundation

/// Observable wrapper around the Rust engine. All UI state derives from engine
/// commands — the app never holds product truth, it just renders the engine's.
///
/// Method/type names (`KansoEngine`, `NoteDto`, `createNote(notebookId:...)`, …)
/// come from the generated `kanso_ffi.swift`.
@MainActor
final class KansoStore: ObservableObject {
    private let engine: KansoEngine

    @Published var notebooks: [NotebookDto] = []
    @Published var notes: [NoteDto] = []
    @Published var selectedNotebookId: String?
    @Published var selectedNoteId: String?
    @Published var searchQuery: String = ""

    init() {
        do {
            // Production opens a file under Application Support; in-memory keeps
            // the first run zero-config.
            let support = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first
            if let dir = support?.appendingPathComponent("Kanso", isDirectory: true) {
                try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
                let db = dir.appendingPathComponent("kanso.db").path
                self.engine = try KansoEngine.open(path: db)
            } else {
                self.engine = try KansoEngine.openInMemory()
            }
        } catch {
            // A failed engine open is unrecoverable; surface it loudly in dev.
            fatalError("failed to open Kanso engine: \(error)")
        }
        refreshNotebooks()
    }

    // MARK: - Notebooks

    func refreshNotebooks() {
        notebooks = (try? engine.listNotebooks()) ?? []
        if selectedNotebookId == nil { selectedNotebookId = notebooks.first?.id }
        refreshNotes()
    }

    func createNotebook(name: String) {
        _ = try? engine.createNotebook(name: name, parentId: nil)
        refreshNotebooks()
    }

    // MARK: - Notes

    func refreshNotes() {
        guard let nb = selectedNotebookId else { notes = []; return }
        if searchQuery.isEmpty {
            notes = (try? engine.listNotes(notebookId: nb)) ?? []
        } else {
            notes = (try? engine.searchNotes(query: searchQuery)) ?? []
        }
        if !notes.contains(where: { $0.id == selectedNoteId }) {
            selectedNoteId = notes.first?.id
        }
    }

    func createNote() {
        guard let nb = selectedNotebookId else { return }
        if let note = try? engine.createNote(notebookId: nb, title: "Untitled", bodyMarkdown: "") {
            selectedNoteId = note.id
        }
        refreshNotes()
    }

    func updateBody(noteId: String, body: String) {
        try? engine.updateNoteBody(noteId: noteId, bodyMarkdown: body)
        refreshNotes()
    }

    func deleteNote(noteId: String) {
        try? engine.deleteNote(noteId: noteId)
        refreshNotes()
    }

    func note(_ id: String?) -> NoteDto? {
        guard let id else { return nil }
        return (try? engine.getNote(noteId: id)) ?? nil
    }
}
