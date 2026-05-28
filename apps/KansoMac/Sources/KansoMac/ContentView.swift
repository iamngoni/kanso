import SwiftUI

/// The three-pane shell: notebooks | note list | editor. Restrained and native,
/// per the macOS Desktop Design Spec — the note is the main object.
struct ContentView: View {
    @EnvironmentObject private var store: KansoStore

    var body: some View {
        NavigationSplitView {
            sidebar
        } content: {
            noteList
        } detail: {
            editor
        }
    }

    // MARK: Sidebar — notebooks

    private var sidebar: some View {
        List(selection: Binding(
            get: { store.selectedNotebookId },
            set: { store.selectedNotebookId = $0; store.refreshNotes() }
        )) {
            Section("Notebooks") {
                ForEach(store.notebooks, id: \.id) { nb in
                    Label(nb.name, systemImage: "book.closed").tag(nb.id)
                }
            }
        }
        .navigationSplitViewColumnWidth(min: 200, ideal: 220, max: 280)
        .toolbar {
            Button { store.createNotebook(name: "New Notebook") } label: {
                Image(systemName: "plus")
            }
        }
    }

    // MARK: Note list

    private var noteList: some View {
        List(selection: $store.selectedNoteId) {
            ForEach(store.notes, id: \.id) { note in
                VStack(alignment: .leading, spacing: 2) {
                    Text(note.title.isEmpty ? "Untitled" : note.title)
                        .font(.system(size: 13, weight: .semibold))
                    Text(note.bodyMarkdown.prefix(80))
                        .font(.system(size: 11))
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }
                .tag(note.id)
                .contextMenu {
                    Button("Delete", role: .destructive) { store.deleteNote(noteId: note.id) }
                }
            }
        }
        .navigationSplitViewColumnWidth(min: 260, ideal: 320, max: 420)
        .searchable(text: $store.searchQuery, prompt: "Search notes")
        .onChange(of: store.searchQuery) { store.refreshNotes() }
        .toolbar {
            Button { store.createNote() } label: { Image(systemName: "square.and.pencil") }
        }
    }

    // MARK: Editor

    private var editor: some View {
        Group {
            if let note = store.note(store.selectedNoteId) {
                NoteEditor(note: note)
                    .id(note.id)
            } else {
                ContentUnavailableView("No note selected", systemImage: "doc.text",
                                       description: Text("Create a note or pick one from the list."))
            }
        }
    }
}

/// A minimal Markdown editor bound to one note. Writes flow straight back into
/// the engine, which re-indexes and snapshots a revision.
private struct NoteEditor: View {
    @EnvironmentObject private var store: KansoStore
    let note: NoteDto
    @State private var body_: String = ""

    var body: some View {
        TextEditor(text: $body_)
            .font(.system(size: 16, design: .default))
            .lineSpacing(4)
            .padding(24)
            .onAppear { body_ = note.bodyMarkdown }
            .onChange(of: body_) { _, newValue in
                store.updateBody(noteId: note.id, body: newValue)
            }
            .navigationTitle(note.title.isEmpty ? "Untitled" : note.title)
    }
}
