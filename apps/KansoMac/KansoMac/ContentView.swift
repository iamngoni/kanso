import SwiftUI

/// Three-pane shell — sidebar | note list | editor — styled to the macOS Desktop
/// Design Spec: warm graphite, off-white text, restrained blue-gray accent.
struct ContentView: View {
    @EnvironmentObject var store: KansoStore

    var body: some View {
        NavigationSplitView {
            SidebarView()
                .navigationSplitViewColumnWidth(min: 200, ideal: 220, max: 260)
        } content: {
            NoteListView()
                .navigationSplitViewColumnWidth(min: 280, ideal: 320, max: 420)
        } detail: {
            EditorView()
        }
        .navigationSplitViewStyle(.balanced)
        .preferredColorScheme(.dark)
    }
}

// MARK: - Sidebar

private struct SidebarView: View {
    @EnvironmentObject var store: KansoStore
    @State private var showingNewNotebook = false
    @State private var newNotebookName = ""

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            Text("Kanso")
                .font(.system(size: 15, weight: .semibold))
                .foregroundStyle(Theme.textPrimary)
                .padding(.horizontal, 16)
                .padding(.top, 18)
                .padding(.bottom, 12)

            ScrollView {
                VStack(alignment: .leading, spacing: 1) {
                    SidebarRow(icon: "tray.full", label: "All Notes", count: store.allNotes.count,
                               selected: store.selection == .all) { store.select(.all) }
                    SidebarRow(icon: "pin", label: "Pinned", count: store.pinnedCount,
                               selected: store.selection == .pinned) { store.select(.pinned) }
                    SidebarRow(icon: "clock", label: "Recent", count: nil,
                               selected: store.selection == .recent) { store.select(.recent) }

                    notebooksHeader
                    ForEach(store.notebooks, id: \.id) { nb in
                        SidebarRow(icon: "book.closed", label: nb.name,
                                   count: store.noteCount(forNotebook: nb.id),
                                   selected: store.selection == .notebook(nb.id)) {
                            store.select(.notebook(nb.id))
                        }
                    }

                    if !store.tags.isEmpty {
                        sectionLabel("TAGS")
                        ForEach(store.tags, id: \.id) { tag in
                            HStack(spacing: 8) {
                                Circle().fill(Theme.accent).frame(width: 7, height: 7)
                                Text(tag.name).font(.system(size: 12)).foregroundStyle(Theme.textSecondary)
                                Spacer()
                            }
                            .padding(.horizontal, 18)
                            .padding(.vertical, 4)
                        }
                    }
                }
                .padding(.vertical, 4)
            }

            Spacer(minLength: 0)
            Divider().overlay(Theme.divider)
            HStack(spacing: 9) {
                Circle().fill(Theme.accent).frame(width: 22, height: 22)
                    .overlay(Text("M").font(.system(size: 11, weight: .semibold)).foregroundStyle(.white))
                VStack(alignment: .leading, spacing: 1) {
                    Text("modestnerd").font(.system(size: 12, weight: .medium)).foregroundStyle(Theme.textPrimary)
                    Text("Local · synced").font(.system(size: 10)).foregroundStyle(Theme.textMuted)
                }
                Spacer()
                Image(systemName: "gearshape").font(.system(size: 12)).foregroundStyle(Theme.textMuted)
            }
            .padding(.horizontal, 14)
            .padding(.vertical, 10)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .background(Theme.sidebar)
        .alert("New Notebook", isPresented: $showingNewNotebook) {
            TextField("Name", text: $newNotebookName)
            Button("Create") {
                let name = newNotebookName.trimmingCharacters(in: .whitespaces)
                if !name.isEmpty { store.createNotebook(name: name) }
                newNotebookName = ""
            }
            Button("Cancel", role: .cancel) { newNotebookName = "" }
        } message: {
            Text("Name your notebook.")
        }
    }

    private var notebooksHeader: some View {
        HStack(spacing: 0) {
            Text("NOTEBOOKS")
                .font(.system(size: 10, weight: .semibold))
                .foregroundStyle(Theme.textMuted)
            Spacer()
            Button { showingNewNotebook = true } label: {
                Image(systemName: "plus")
                    .font(.system(size: 11, weight: .medium))
                    .foregroundStyle(Theme.textMuted)
            }
            .buttonStyle(.plain)
            .help("New notebook")
        }
        .padding(.horizontal, 16)
        .padding(.top, 16)
        .padding(.bottom, 4)
    }

    private func sectionLabel(_ text: String) -> some View {
        Text(text)
            .font(.system(size: 10, weight: .semibold))
            .foregroundStyle(Theme.textMuted)
            .padding(.horizontal, 16)
            .padding(.top, 16)
            .padding(.bottom, 4)
    }
}

private struct SidebarRow: View {
    let icon: String
    let label: String
    let count: Int?
    let selected: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 9) {
                Image(systemName: icon)
                    .font(.system(size: 12))
                    .frame(width: 16)
                    .foregroundStyle(selected ? Theme.accent : Theme.textMuted)
                Text(label)
                    .font(.system(size: 13))
                    .foregroundStyle(selected ? Theme.textPrimary : Theme.textSecondary)
                    .lineLimit(1)
                Spacer()
                if let count, count > 0 {
                    Text("\(count)").font(.system(size: 11)).foregroundStyle(Theme.textMuted)
                }
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 5)
            .background(RoundedRectangle(cornerRadius: 6)
                .fill(selected ? Theme.accent.opacity(0.18) : Color.clear))
            .padding(.horizontal, 8)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
    }
}

// MARK: - Note list

private struct NoteListView: View {
    @EnvironmentObject var store: KansoStore

    var body: some View {
        VStack(spacing: 0) {
            HStack(spacing: 5) {
                Text(store.currentTitle).font(.system(size: 13, weight: .semibold)).foregroundStyle(Theme.textPrimary)
                Image(systemName: "chevron.down").font(.system(size: 9)).foregroundStyle(Theme.textMuted)
                Spacer()
                Button { store.createNote() } label: {
                    Image(systemName: "square.and.pencil").font(.system(size: 13)).foregroundStyle(Theme.textSecondary)
                }
                .buttonStyle(.plain)
            }
            .padding(.horizontal, 14)
            .padding(.top, 16)
            .padding(.bottom, 10)

            HStack(spacing: 6) {
                Image(systemName: "magnifyingglass").font(.system(size: 11)).foregroundStyle(Theme.textMuted)
                TextField("Search notes", text: $store.search)
                    .textFieldStyle(.plain)
                    .font(.system(size: 12))
                    .foregroundStyle(Theme.textPrimary)
                    .onChange(of: store.search) { store.recompute() }
            }
            .padding(.horizontal, 9)
            .padding(.vertical, 6)
            .background(RoundedRectangle(cornerRadius: 7).fill(Theme.elevated))
            .padding(.horizontal, 12)
            .padding(.bottom, 8)

            if store.notes.isEmpty {
                VStack(spacing: 6) {
                    Image(systemName: "tray").font(.system(size: 20)).foregroundStyle(Theme.textMuted)
                    Text("No notes").font(.system(size: 12)).foregroundStyle(Theme.textMuted)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ScrollView {
                    LazyVStack(spacing: 0) {
                        ForEach(store.notes, id: \.id) { note in
                            NoteRow(note: note, selected: note.id == store.selectedNoteId) {
                                store.selectedNoteId = note.id
                            }
                        }
                    }
                }
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Theme.noteList)
    }
}

private struct NoteRow: View {
    let note: NoteDto
    let selected: Bool
    let action: () -> Void

    var body: some View {
        VStack(spacing: 0) {
            Button(action: action) {
                VStack(alignment: .leading, spacing: 3) {
                    HStack(spacing: 6) {
                        if note.pinned {
                            Image(systemName: "pin.fill").font(.system(size: 9)).foregroundStyle(Theme.accent)
                        }
                        Text(note.title.isEmpty ? "Untitled" : note.title)
                            .font(.system(size: 13, weight: .semibold))
                            .foregroundStyle(Theme.textPrimary)
                            .lineLimit(1)
                    }
                    Text(excerpt(note.bodyMarkdown))
                        .font(.system(size: 11)).foregroundStyle(Theme.textMuted).lineLimit(1)
                    Text(shortDate(note.updatedAt))
                        .font(.system(size: 10)).foregroundStyle(Theme.textMuted)
                }
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.horizontal, 14)
                .padding(.vertical, 9)
                .background(selected ? Theme.elevated : Color.clear)
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)
            Divider().overlay(Theme.divider.opacity(0.5)).padding(.leading, 14)
        }
    }

    private func excerpt(_ body: String) -> String {
        let flat = body.replacingOccurrences(of: "\n", with: " ").trimmingCharacters(in: .whitespaces)
        return flat.isEmpty ? "No additional text" : flat
    }
}

// MARK: - Editor

private struct EditorView: View {
    @EnvironmentObject var store: KansoStore

    var body: some View {
        Group {
            if let note = store.note(store.selectedNoteId) {
                NoteEditor(note: note).id(note.id)
            } else {
                EmptyEditorState()
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Theme.editor)
    }
}

private struct EmptyEditorState: View {
    @EnvironmentObject var store: KansoStore

    var body: some View {
        VStack(spacing: 13) {
            Image(systemName: "doc.text").font(.system(size: 30)).foregroundStyle(Theme.textMuted)
            Text("No notes in \(store.currentTitle)")
                .font(.system(size: 17, weight: .semibold)).foregroundStyle(Theme.textPrimary)
            Text("Get started by creating a new note.\nMarkdown notes stay portable.")
                .font(.system(size: 12)).foregroundStyle(Theme.textMuted).multilineTextAlignment(.center)
            Button { store.createNote() } label: {
                Text("New note")
                    .font(.system(size: 13, weight: .medium))
                    .foregroundStyle(.white)
                    .padding(.horizontal, 16)
                    .padding(.vertical, 8)
                    .background(RoundedRectangle(cornerRadius: 7).fill(Theme.accent))
            }
            .buttonStyle(.plain)
            .padding(.top, 2)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

private struct NoteEditor: View {
    @EnvironmentObject var store: KansoStore
    let note: NoteDto
    @State private var draft: String = ""

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack {
                Text(note.title.isEmpty ? "Untitled" : note.title)
                    .font(.system(size: 18, weight: .semibold))
                    .foregroundStyle(Theme.textPrimary)
                Spacer()
                Button { store.deleteNote(note.id) } label: {
                    Image(systemName: "trash").font(.system(size: 13)).foregroundStyle(Theme.textMuted)
                }
                .buttonStyle(.plain)
            }
            .padding(.horizontal, 26)
            .padding(.top, 22)
            .padding(.bottom, 10)

            Divider().overlay(Theme.divider)

            TextEditor(text: $draft)
                .font(.system(size: 16))
                .foregroundStyle(Theme.textPrimary)
                .scrollContentBackground(.hidden)
                .background(Theme.editor)
                .padding(.horizontal, 22)
                .padding(.top, 10)
                .onChange(of: draft) { _, newValue in store.updateBody(note.id, newValue) }
        }
        .onAppear { draft = note.bodyMarkdown }
    }
}

// MARK: - Helpers

private func shortDate(_ millis: Int64) -> String {
    let date = Date(timeIntervalSince1970: Double(millis) / 1000)
    let formatter = DateFormatter()
    formatter.dateFormat = "MMM d"
    return formatter.string(from: date)
}
