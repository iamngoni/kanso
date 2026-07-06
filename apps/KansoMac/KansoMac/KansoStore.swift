import Foundation
import Combine

/// Observable wrapper around the Rust engine. The app holds no product truth —
/// it renders the engine's. Library views (All/Pinned/Recent) are derived
/// client-side from the aggregated note set (each `NoteDto` carries `pinned`
/// and `updatedAt`); notebooks filter the same set.
@MainActor
final class KansoStore: ObservableObject {
    enum SidebarSelection: Equatable {
        case all
        case pinned
        case recent
        case notebook(String)
    }

    private let engine: KansoEngine

    @Published var notebooks: [NotebookDto] = []
    @Published var tags: [TagDto] = []
    @Published var allNotes: [NoteDto] = []
    @Published var notes: [NoteDto] = []
    @Published var selection: SidebarSelection = .all
    @Published var selectedNoteId: String?
    @Published var search: String = ""

    init() {
        do {
            let support = FileManager.default
                .urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
            let dir = support.appendingPathComponent("Kanso", isDirectory: true)
            try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
            engine = try KansoEngine.open(path: dir.appendingPathComponent("kanso.db").path)
        } catch {
            fatalError("failed to open Kanso engine: \(error)")
        }
        reload()
    }

    // MARK: Derived

    var currentTitle: String {
        switch selection {
        case .all: return "All Notes"
        case .pinned: return "Pinned"
        case .recent: return "Recent"
        case .notebook(let id): return notebooks.first { $0.id == id }?.name ?? "Notebook"
        }
    }

    var pinnedCount: Int { allNotes.filter { $0.pinned }.count }

    func noteCount(forNotebook id: String) -> Int {
        allNotes.filter { $0.notebookId == id }.count
    }

    // MARK: Loading

    func reload() {
        notebooks = (try? engine.listNotebooks()) ?? []
        tags = (try? engine.listTags()) ?? []
        var acc: [NoteDto] = []
        for nb in notebooks {
            acc.append(contentsOf: (try? engine.listNotes(notebookId: nb.id)) ?? [])
        }
        allNotes = acc
        recompute()
    }

    func recompute() {
        if !search.isEmpty {
            notes = (try? engine.searchNotes(query: search)) ?? []
        } else {
            let sorted = allNotes.sorted { $0.updatedAt > $1.updatedAt }
            switch selection {
            case .all: notes = sorted
            case .pinned: notes = sorted.filter { $0.pinned }
            case .recent: notes = Array(sorted.prefix(20))
            case .notebook(let id): notes = sorted.filter { $0.notebookId == id }
            }
        }
        if !notes.contains(where: { $0.id == selectedNoteId }) {
            selectedNoteId = notes.first?.id
        }
    }

    func select(_ selection: SidebarSelection) {
        self.selection = selection
        recompute()
    }

    // MARK: Mutations

    func createNotebook(name: String) {
        guard let nb = try? engine.createNotebook(name: name, parentId: nil) else { return }
        reload()
        select(.notebook(nb.id))
    }

    func createNote() {
        let targetNotebook: String? = {
            if case .notebook(let id) = selection { return id }
            return notebooks.first?.id
        }()
        let notebookId: String
        if let existing = targetNotebook {
            notebookId = existing
        } else {
            // Fresh library — no notebooks yet. Create a default one so
            // "New note" is never a dead end on first launch.
            guard let nb = try? engine.createNotebook(name: "Notes", parentId: nil) else { return }
            notebookId = nb.id
        }
        if let note = try? engine.createNote(notebookId: notebookId, title: "Untitled", bodyMarkdown: "") {
            selection = .notebook(notebookId)
            reload()
            selectedNoteId = note.id
        }
    }

    func updateBody(_ noteId: String, _ body: String) {
        try? engine.updateNoteBody(noteId: noteId, bodyMarkdown: body)
        if let updated = (try? engine.getNote(noteId: noteId)) ?? nil,
           let index = allNotes.firstIndex(where: { $0.id == noteId }) {
            allNotes[index] = updated
        }
    }

    func deleteNote(_ noteId: String) {
        try? engine.deleteNote(noteId: noteId)
        reload()
    }

    func note(_ id: String?) -> NoteDto? {
        guard let id else { return nil }
        return (try? engine.getNote(noteId: id)) ?? nil
    }
}
