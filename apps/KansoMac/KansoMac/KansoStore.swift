import Foundation
import Combine
import CryptoKit
import Security
import UniformTypeIdentifiers

enum KansoEditorMode: String, CaseIterable, Identifiable {
    case edit
    case preview
    case split

    var id: String { rawValue }

    var symbol: String {
        switch self {
        case .edit: return "pencil"
        case .preview: return "doc.richtext"
        case .split: return "rectangle.split.2x1"
        }
    }

    var title: String {
        switch self {
        case .edit: return "Edit"
        case .preview: return "Preview"
        case .split: return "Split"
        }
    }
}

enum KansoSyncPhase: Equatable {
    case localOnly
    case ready
    case syncing
    case error

    var title: String {
        switch self {
        case .localOnly: return "Local only"
        case .ready: return "Ready"
        case .syncing: return "Syncing"
        case .error: return "Needs attention"
        }
    }
}

enum KansoEditorAction: Equatable {
    case insertSketch
    case attachFile
    case showHistory
    case shareNote
}

struct NotebookOutlineItem: Identifiable {
    let notebook: NotebookDto
    let depth: Int

    var id: String { notebook.id }
}

/// Observable wrapper around the Rust engine. The app holds no product truth —
/// it renders the engine's. Library views (All/Pinned/Recent) are derived
/// client-side from the aggregated note set (each `NoteDto` carries `pinned`
/// and `updatedAt`); notebooks filter the same set.
@MainActor
final class KansoStore: ObservableObject {
    static let defaultSyncBaseURL = "http://127.0.0.1:8787"
    private static let legacySyncBaseURLs: Set<String> = [
        "http://localhost:8793",
        "http://127.0.0.1:8793"
    ]
    private static let backupEncryptionService = "za.co.codecraftsolutions.KansoMac.backup-encryption"
    private static let backupEncryptionAccount = "default"

    enum SidebarSelection: Equatable {
        case all
        case pinned
        case recent
        case tasks
        case notebook(String)
        case tag(String)
        case trash
    }

    private var engine: KansoEngine

    @Published var notebooks: [NotebookDto] = []
    @Published var tags: [TagDto] = []
    @Published var allNotes: [NoteDto] = []
    @Published var openTaskItems: [TaskItemDto] = []
    @Published var trashNotes: [NoteDto] = []
    @Published var notes: [NoteDto] = []
    @Published var selection: SidebarSelection = .all
    @Published var selectedNoteId: String?
    @Published var search: String = ""
    @Published var editorMode: KansoEditorMode = .split
    @Published var syncBaseURL: String = KansoStore.defaultSyncBaseURL
    @Published var syncEmail: String = ""
    @Published var syncUserId: String = ""
    @Published var syncDeviceId: String = ""
    @Published var syncEncryptionEnabled: Bool = false
    @Published var syncEncryptionUnlocked: Bool = false
    @Published var syncPhase: KansoSyncPhase = .localOnly
    @Published var syncMessage: String = "Local notes only"
    @Published var isSyncing: Bool = false
    @Published var isCommandPalettePresented = false
    @Published var isSettingsPresented = false
    @Published var requestedSettingsSection: String?
    @Published var pendingEditorAction: KansoEditorAction?
    @Published var mcpClients: [McpClientDto] = []
    @Published var mcpCapabilities: [String: Set<String>] = [:]
    @Published var skills: [SkillDto] = []
    @Published var skillRuns: [String: [SkillRunDto]] = [:]
    @Published var settingsMessage: String = ""

    private var syncToken: String = ""
    private var syncEncryptionSalt: String = ""
    private var pendingAutoSyncTask: Task<Void, Never>?
    private var syncAgainAfterCurrentRun = false

    let mcpCapabilityOptions = ["read", "write", "delete", "run_skill"]

    init() {
        do {
            let path = try Self.databasePath()
            let encryption = Self.loadBackupEncryptionSettings()
            let openedEngine: KansoEngine
            let unlocked: Bool
            if encryption.enabled,
               let passphrase = Self.loadBackupEncryptionPassphrase(),
               !encryption.salt.isEmpty {
                openedEngine = try KansoEngine.openWithEncryptionPassphrase(
                    path: path,
                    passphrase: passphrase,
                    salt: encryption.salt
                )
                unlocked = true
            } else {
                openedEngine = try KansoEngine.open(path: path)
                unlocked = false
            }
            engine = openedEngine
            syncEncryptionEnabled = encryption.enabled
            syncEncryptionSalt = encryption.salt
            syncEncryptionUnlocked = unlocked
        } catch {
            fatalError("failed to open Kanso engine: \(error)")
        }
        loadSyncSettings()
        reload()
        scheduleAutoSync(reason: "Backup check queued", delayNanoseconds: 1_000_000_000)
    }

    // MARK: Derived

    var currentTitle: String {
        switch selection {
        case .all: return "All Notes"
        case .pinned: return "Pinned"
        case .recent: return "Recent"
        case .tasks: return "Tasks"
        case .notebook(let id): return notebooks.first { $0.id == id }?.name ?? "Notebook"
        case .tag(let id): return tags.first { $0.id == id }?.name ?? "Tag"
        case .trash: return "Trash"
        }
    }

    var pinnedCount: Int { allNotes.filter { $0.pinned }.count }
    var trashCount: Int { trashNotes.count }

    var isSyncAuthenticated: Bool {
        !syncToken.isEmpty && !syncDeviceId.isEmpty
    }

    var isSyncConfigured: Bool {
        isSyncAuthenticated && backupEncryptionAllowsSync
    }

    private var backupEncryptionAllowsSync: Bool {
        syncEncryptionEnabled && syncEncryptionUnlocked
    }

    var backupEncryptionStatus: String {
        if !syncEncryptionEnabled { return "Off" }
        return syncEncryptionUnlocked ? "Unlocked" : "Locked"
    }

    var backupEncryptionActionTitle: String {
        if syncEncryptionEnabled, syncEncryptionUnlocked { return "Encryption Unlocked" }
        return syncEncryptionEnabled ? "Unlock Encryption" : "Enable Encryption"
    }

    var syncStatusLine: String {
        if syncPhase != .syncing, isSyncAuthenticated, !backupEncryptionAllowsSync {
            return syncEncryptionEnabled ? "Encryption locked" : "Encryption required"
        }
        switch syncPhase {
        case .syncing:
            return "Syncing..."
        case .localOnly:
            return "Local only"
        case .ready, .error:
            return syncMessage
        }
    }

    var syncSettingsDetailLine: String {
        if isSyncAuthenticated, !backupEncryptionAllowsSync {
            return syncEncryptionEnabled
                ? "Unlock backup encryption to sync."
                : "Enable backup encryption before syncing."
        }
        if syncPhase == .syncing || syncPhase == .error || syncMessage != "Local notes only" {
            return syncMessage
        }
        return "Connect to Kanso Cloud or a local Wrangler server."
    }

    func noteCount(forNotebook id: String) -> Int {
        allNotes.filter { $0.notebookId == id }.count
    }

    func recursiveNoteCount(forNotebook id: String) -> Int {
        let childIds = childNotebooks(of: id).map(\.id)
        return noteCount(forNotebook: id)
            + childIds.reduce(0) { $0 + recursiveNoteCount(forNotebook: $1) }
    }

    var rootNotebooks: [NotebookDto] {
        notebooks.filter { $0.parentId == nil }
    }

    var notebookOutline: [NotebookOutlineItem] {
        var rows: [NotebookOutlineItem] = []
        for notebook in rootNotebooks {
            appendNotebook(notebook, depth: 0, into: &rows)
        }
        return rows
    }

    func childNotebooks(of parentId: String) -> [NotebookDto] {
        notebooks.filter { $0.parentId == parentId }
    }

    func canMoveNotebook(_ id: String, under parentId: String?) -> Bool {
        guard parentId != id else { return false }
        guard let parentId else { return true }
        return !isDescendant(parentId, of: id)
    }

    func notebookName(_ id: String) -> String {
        notebooks.first { $0.id == id }?.name ?? "Notebook"
    }

    func tagCount(_ tagId: String) -> Int {
        ((try? engine.notesWithTag(tagId: tagId)) ?? []).count
    }

    // MARK: Loading

    func reload() {
        notebooks = (try? engine.listNotebooks()) ?? []
        tags = (try? engine.listTags()) ?? []
        var acc: [NoteDto] = []
        for nb in notebooks {
            acc.append(contentsOf: (try? engine.listNotes(notebookId: nb.id)) ?? [])
        }
        if promotePlaceholderTitles(in: acc) {
            acc = []
            for nb in notebooks {
                acc.append(contentsOf: (try? engine.listNotes(notebookId: nb.id)) ?? [])
            }
        }
        allNotes = acc
        openTaskItems = notebooks.flatMap { (try? engine.listOpenTasks(notebookId: $0.id)) ?? [] }
        trashNotes = (try? engine.listTrash()) ?? []
        reloadAgentSettings()
        recompute()
    }

    func recompute() {
        if !search.isEmpty {
            let needle = search.localizedLowercase
            switch selection {
            case .trash:
                notes = trashNotes.filter { note in
                    note.title.localizedLowercase.contains(needle)
                        || note.bodyMarkdown.localizedLowercase.contains(needle)
                }
            case .tasks:
                let taskNoteIds = Set(openTaskItems.filter { task in
                    task.text.localizedLowercase.contains(needle)
                }.map(\.noteId))
                let taskNotes = notesWithOpenTasks(from: allNotes)
                let bodyMatches = taskNotes.filter { note in
                    note.title.localizedLowercase.contains(needle)
                        || note.bodyMarkdown.localizedLowercase.contains(needle)
                }
                let taskMatches = taskNotes.filter { taskNoteIds.contains($0.id) }
                notes = Array((bodyMatches + taskMatches).reduce(into: [String: NoteDto]()) { $0[$1.id] = $1 }.values)
                    .sorted { $0.updatedAt > $1.updatedAt }
            default:
                notes = (try? engine.searchNotes(query: search)) ?? []
            }
        } else {
            let sorted = allNotes.sorted { $0.updatedAt > $1.updatedAt }
            switch selection {
            case .all: notes = sorted
            case .pinned: notes = sorted.filter { $0.pinned }
            case .recent: notes = Array(sorted.prefix(20))
            case .tasks: notes = notesWithOpenTasks(from: sorted)
            case .notebook(let id): notes = sorted.filter { $0.notebookId == id }
            case .tag(let id): notes = ((try? engine.notesWithTag(tagId: id)) ?? [])
            case .trash: notes = trashNotes.sorted { $0.updatedAt > $1.updatedAt }
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

    func createNotebook(name: String, parentId: String? = nil) {
        guard let nb = try? engine.createNotebook(name: name, parentId: parentId) else { return }
        reload()
        select(.notebook(nb.id))
        scheduleAutoSync(reason: "Notebook will back up shortly")
    }

    func renameNotebook(_ id: String, name: String) {
        let trimmed = name.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return }
        do {
            try engine.renameNotebook(notebookId: id, name: trimmed)
            reload()
            select(.notebook(id))
            scheduleAutoSync(reason: "Notebook rename will back up shortly")
        } catch {
            syncPhase = .error
            syncMessage = Self.userFacingOperationError(error, fallback: "Rename failed")
        }
    }

    func moveNotebook(_ id: String, parentId: String?) {
        do {
            try engine.moveNotebook(notebookId: id, parentId: parentId)
            reload()
            select(.notebook(id))
            scheduleAutoSync(reason: "Notebook move will back up shortly")
        } catch {
            syncPhase = .error
            syncMessage = Self.userFacingOperationError(error, fallback: "Move failed")
        }
    }

    @discardableResult
    func deleteNotebook(_ id: String) -> Bool {
        guard recursiveNoteCount(forNotebook: id) == 0,
              childNotebooks(of: id).isEmpty else {
            syncPhase = .error
            syncMessage = "Move notes and child notebooks before deleting"
            return false
        }

        do {
            try engine.deleteNotebook(notebookId: id)
            if selection == .notebook(id) {
                selection = .all
            }
            reload()
            scheduleAutoSync(reason: "Notebook deletion will back up shortly")
            return true
        } catch {
            syncPhase = .error
            syncMessage = Self.userFacingOperationError(error, fallback: "Delete failed")
            return false
        }
    }

    func createNote(title: String = "Untitled", bodyMarkdown: String = "") {
        let initialTitle = Self.effectiveTitle(requestedTitle: title, bodyMarkdown: bodyMarkdown)
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
        if let note = try? engine.createNote(notebookId: notebookId, title: initialTitle, bodyMarkdown: bodyMarkdown) {
            selection = .notebook(notebookId)
            reload()
            selectedNoteId = note.id
            scheduleAutoSync(reason: "Note will back up shortly")
        }
    }

    func openNote(_ noteId: String) {
        selection = .all
        recompute()
        selectedNoteId = noteId
    }

    func currentNotebookId() -> String? {
        if case .notebook(let id) = selection {
            return id
        }
        if let selected = note(selectedNoteId) {
            return selected.notebookId
        }
        return notebooks.first?.id
    }

    func createDailyNote() {
        let targetNotebook: String? = {
            if case .notebook(let id) = selection { return id }
            return notebooks.first?.id
        }()
        let notebookId: String
        if let existing = targetNotebook {
            notebookId = existing
        } else {
            guard let nb = try? engine.createNotebook(name: "Notes", parentId: nil) else { return }
            notebookId = nb.id
        }
        if let note = try? engine.createDailyNote(notebookId: notebookId) {
            selection = .notebook(notebookId)
            reload()
            selectedNoteId = note.id
            scheduleAutoSync(reason: "Daily note will back up shortly")
        }
    }

    func updateBody(_ noteId: String, _ body: String) {
        guard let current = note(noteId), current.bodyMarkdown != body else { return }
        try? engine.updateNoteBody(noteId: noteId, bodyMarkdown: body)
        if shouldPromotePlaceholderTitle(current.title),
           let title = Self.derivedTitle(fromMarkdown: body),
           title != current.title {
            try? engine.renameNote(noteId: noteId, title: title)
        }
        replaceNoteIfPresent(noteId)
        scheduleAutoSync(reason: "Changes will back up shortly")
    }

    func updateTitle(_ noteId: String, _ title: String) {
        let trimmed = title.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return }
        try? engine.renameNote(noteId: noteId, title: trimmed)
        replaceNoteIfPresent(noteId)
        scheduleAutoSync(reason: "Title change will back up shortly")
    }

    func moveNote(_ noteId: String, toNotebook notebookId: String) {
        do {
            try engine.moveNote(noteId: noteId, notebookId: notebookId)
            reload()
            selectedNoteId = noteId
            scheduleAutoSync(reason: "Note move will back up shortly")
        } catch {
            syncPhase = .error
            syncMessage = Self.userFacingOperationError(error, fallback: "Move failed")
        }
    }

    func setPinned(_ noteId: String, pinned: Bool) {
        try? engine.setNotePinned(noteId: noteId, pinned: pinned)
        replaceNoteIfPresent(noteId)
        recompute()
        scheduleAutoSync(reason: "Pin change will back up shortly")
    }

    func setFavorite(_ noteId: String, favorite: Bool) {
        try? engine.setNoteFavorite(noteId: noteId, favorite: favorite)
        replaceNoteIfPresent(noteId)
        recompute()
        scheduleAutoSync(reason: "Favorite change will back up shortly")
    }

    func setStatus(_ noteId: String, status: String) {
        try? engine.setNoteStatus(noteId: noteId, status: status)
        replaceNoteIfPresent(noteId)
        recompute()
        scheduleAutoSync(reason: "Status change will back up shortly")
    }

    func renderHTML(_ noteId: String) -> String {
        (try? engine.renderNoteHtml(noteId: noteId)) ?? "<p>Preview unavailable.</p>"
    }

    func tags(forNote noteId: String) -> [TagDto] {
        (try? engine.tagsForNote(noteId: noteId)) ?? []
    }

    func createTag(named name: String) -> TagDto? {
        let trimmed = name.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return nil }
        if let existing = tags.first(where: { $0.name.caseInsensitiveCompare(trimmed) == .orderedSame }) {
            return existing
        }
        guard let tag = try? engine.createTag(name: trimmed) else { return nil }
        reload()
        scheduleAutoSync(reason: "Tag will back up shortly")
        return tag
    }

    func setTag(_ tag: TagDto, on noteId: String, enabled: Bool) {
        if enabled {
            try? engine.tagNote(noteId: noteId, tagId: tag.id)
        } else {
            try? engine.untagNote(noteId: noteId, tagId: tag.id)
        }
        reload()
        selectedNoteId = noteId
        scheduleAutoSync(reason: "Tag change will back up shortly")
    }

    func backlinks(forNote noteId: String) -> [NoteDto] {
        (try? engine.backlinks(noteId: noteId)) ?? []
    }

    func outgoingLinks(forNote noteId: String) -> [NoteLinkDto] {
        (try? engine.outgoingLinks(noteId: noteId)) ?? []
    }

    @discardableResult
    func openOrCreateLinkedNote(named target: String, preferredNotebookId: String?) -> NoteDto? {
        guard let title = Self.canonicalWikiTarget(target) else { return nil }
        if let existing = allNotes.first(where: { note in
            note.id == title || note.title.caseInsensitiveCompare(title) == .orderedSame
        }) {
            openNote(existing.id)
            return existing
        }

        let notebookId: String
        if let preferredNotebookId,
           notebooks.contains(where: { $0.id == preferredNotebookId }) {
            notebookId = preferredNotebookId
        } else if let current = currentNotebookId() {
            notebookId = current
        } else if let nb = try? engine.createNotebook(name: "Notes", parentId: nil) {
            notebookId = nb.id
        } else {
            return nil
        }

        guard let note = try? engine.createNote(notebookId: notebookId, title: title, bodyMarkdown: "") else {
            return nil
        }
        selection = .notebook(notebookId)
        reload()
        selectedNoteId = note.id
        scheduleAutoSync(reason: "Linked note will back up shortly")
        return note
    }

    func attachment(matching target: String, noteId: String) -> AttachmentDto? {
        let canonical = target
            .removingPercentEncoding?
            .trimmingCharacters(in: .whitespacesAndNewlines)
            ?? target.trimmingCharacters(in: .whitespacesAndNewlines)
        let suffix = canonical.hasPrefix("attachment:")
            ? String(canonical.dropFirst("attachment:".count))
            : canonical
        return attachments(forNote: noteId).first { attachment in
            attachment.id == canonical
                || attachment.id == "attachment:\(suffix)"
                || attachment.filename == canonical
                || attachment.contentHash == canonical
        }
    }

    func revisions(forNote noteId: String) -> [RevisionDto] {
        (try? engine.listRevisions(noteId: noteId)) ?? []
    }

    func conflicts(forNote noteId: String) -> [RevisionDto] {
        (try? engine.listConflicts(noteId: noteId)) ?? []
    }

    func restoreRevision(_ revision: RevisionDto) -> String? {
        do {
            try engine.restoreRevision(noteId: revision.noteId, revisionId: revision.id)
            replaceNoteIfPresent(revision.noteId)
            reload()
            selectedNoteId = revision.noteId
            scheduleAutoSync(reason: "Restored revision will back up shortly")
            return note(revision.noteId)?.bodyMarkdown
        } catch {
            syncPhase = .error
            syncMessage = Self.userFacingOperationError(error, fallback: "Restore failed")
            return nil
        }
    }

    func attachments(forNote noteId: String) -> [AttachmentDto] {
        (try? engine.listAttachments(noteId: noteId)) ?? []
    }

    func sketches(forNote noteId: String) -> [SketchDto] {
        (try? engine.listSketches(noteId: noteId)) ?? []
    }

    func sketch(matching target: String, noteId: String) -> SketchDto? {
        let canonical = canonicalSketchTarget(target)
        let suffix = canonical.hasPrefix("sketch:")
            ? String(canonical.dropFirst("sketch:".count))
            : canonical
        return sketches(forNote: noteId).first { sketch in
            sketch.id == canonical
                || sketch.id == "sketch:\(suffix)"
                || sketch.title == canonical
        }
    }

    func sketchPreviewData(matching target: String, noteId: String) -> Data? {
        let canonical = canonicalSketchTarget(target)
        let suffix = canonical.hasPrefix("sketch:")
            ? String(canonical.dropFirst("sketch:".count))
            : canonical
        let sketchId = sketch(matching: target, noteId: noteId)?.id
            ?? (canonical.hasPrefix("sketch:") ? canonical : "sketch:\(suffix)")
        return try? engine.renderSketchPreview(sketchId: sketchId, width: 960, height: 420)
    }

    func sketchStrokes(matching target: String, noteId: String) -> (SketchDto, [InkStroke])? {
        guard let sketch = sketch(matching: target, noteId: noteId),
              let strokes = try? engine.getSketchStrokes(sketchId: sketch.id) else {
            return nil
        }
        return (sketch, strokes)
    }

    func updateSketch(_ sketchId: String, strokes: [InkStroke], noteId: String) -> Bool {
        guard !strokes.isEmpty else { return false }
        do {
            try engine.updateSketch(sketchId: sketchId, strokes: strokes)
            replaceNoteIfPresent(noteId)
            recompute()
            selectedNoteId = noteId
            scheduleAutoSync(reason: "Sketch edit will back up shortly")
            return true
        } catch {
            syncPhase = .error
            syncMessage = Self.userFacingOperationError(error, fallback: "Sketch update failed")
            return false
        }
    }

    func openTasks(for note: NoteDto) -> [TaskItemDto] {
        ((try? engine.listOpenTasks(notebookId: note.notebookId)) ?? [])
            .filter { $0.noteId == note.id }
    }

    func openTaskCount(for note: NoteDto) -> Int {
        openTaskItems.filter { $0.noteId == note.id }.count
    }

    func setTask(_ task: TaskItemDto, checked: Bool) {
        try? engine.setTaskChecked(taskId: task.id, checked: checked)
        replaceNoteIfPresent(task.noteId)
        reload()
        selectedNoteId = task.noteId
        scheduleAutoSync(reason: "Task change will back up shortly")
    }

    func insertSketch(noteId: String, title: String?, strokes: [InkStroke], bodyMarkdown: String) -> String? {
        guard !strokes.isEmpty,
              let sketch = try? engine.createSketch(noteId: noteId, title: title, strokes: strokes) else {
            return nil
        }

        let trimmed = bodyMarkdown.trimmingCharacters(in: .whitespacesAndNewlines)
        let sketchRef = sketch.id.hasPrefix("sketch:") ? String(sketch.id.dropFirst("sketch:".count)) : sketch.id
        let reference = "![[sketch:\(sketchRef)]]"
        let nextBody = trimmed.isEmpty ? reference : "\(bodyMarkdown.rstrip())\n\n\(reference)"
        try? engine.updateNoteBody(noteId: noteId, bodyMarkdown: nextBody)
        replaceNoteIfPresent(noteId)
        recompute()
        selectedNoteId = noteId
        scheduleAutoSync(reason: "Sketch will back up shortly")
        return nextBody
    }

    private func canonicalSketchTarget(_ target: String) -> String {
        target
            .removingPercentEncoding?
            .trimmingCharacters(in: .whitespacesAndNewlines)
            ?? target.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    func attachFile(url: URL, noteId: String, bodyMarkdown: String) -> String? {
        let didAccess = url.startAccessingSecurityScopedResource()
        defer {
            if didAccess {
                url.stopAccessingSecurityScopedResource()
            }
        }

        do {
            let stored = try Self.copyAttachmentIntoLibrary(url)
            let attachment = try engine.attachFile(
                noteId: noteId,
                input: NewAttachmentDto(
                    filename: stored.filename,
                    mimeType: stored.mimeType,
                    sizeBytes: stored.sizeBytes,
                    contentHash: stored.contentHash,
                    localPath: stored.localURL.path
                )
            )
            let attachmentRef = attachment.id.hasPrefix("attachment:")
                ? String(attachment.id.dropFirst("attachment:".count))
                : attachment.id
            let reference = "![[attachment:\(attachmentRef)]]"
            let trimmed = bodyMarkdown.trimmingCharacters(in: .whitespacesAndNewlines)
            let nextBody = trimmed.isEmpty ? reference : "\(bodyMarkdown.rstrip())\n\n\(reference)"
            try engine.updateNoteBody(noteId: noteId, bodyMarkdown: nextBody)
            replaceNoteIfPresent(noteId)
            recompute()
            selectedNoteId = noteId
            scheduleAutoSync(reason: "Attachment will back up shortly")
            return nextBody
        } catch {
            syncPhase = .error
            syncMessage = Self.userFacingOperationError(error, fallback: "Attachment failed")
            return nil
        }
    }

    func deleteAttachment(_ attachment: AttachmentDto, noteId: String, bodyMarkdown: String) -> String {
        try? engine.deleteAttachment(attachmentId: attachment.id)
        let nextBody = bodyMarkdown.removingAttachmentReferences(for: attachment)
        try? engine.updateNoteBody(noteId: noteId, bodyMarkdown: nextBody)
        replaceNoteIfPresent(noteId)
        recompute()
        selectedNoteId = noteId
        scheduleAutoSync(reason: "Attachment removal will back up shortly")
        return nextBody
    }

    func exportCurrentNotebookMarkdown() -> [ExportFileDto] {
        guard let notebookId = currentNotebookId() else { return [] }
        return (try? engine.exportNotebookMarkdown(notebookId: notebookId)) ?? []
    }

    func exportCurrentNotebook(to directory: URL) -> Int {
        guard let notebookId = currentNotebookId() else { return 0 }
        let exportedNotes = allNotes
            .filter { $0.notebookId == notebookId }
            .sorted { $0.createdAt < $1.createdAt }
        guard !exportedNotes.isEmpty else { return 0 }

        var written = 0
        var usedFilenames = Set<String>()
        for note in exportedNotes {
            let noteAttachments = attachments(forNote: note.id)
            let body = Self.exportBody(note.bodyMarkdown, attachments: noteAttachments)
            let content = """
            ---
            title: \(note.title)
            created: \(note.createdAt)
            updated: \(note.updatedAt)
            ---

            \(body)
            """
            let filename = Self.uniqueMarkdownFilename(for: note.title, usedFilenames: &usedFilenames)
            let url = directory.appendingPathComponent(filename)
            if (try? content.write(to: url, atomically: true, encoding: .utf8)) != nil {
                written += 1
            }

            for attachment in noteAttachments {
                guard let localPath = attachment.localPath else { continue }
                let source = URL(fileURLWithPath: localPath)
                guard FileManager.default.fileExists(atPath: source.path) else { continue }
                let destination = directory.appendingPathComponent(Self.exportPath(for: attachment))
                do {
                    try FileManager.default.createDirectory(
                        at: destination.deletingLastPathComponent(),
                        withIntermediateDirectories: true
                    )
                    if FileManager.default.fileExists(atPath: destination.path) {
                        try FileManager.default.removeItem(at: destination)
                    }
                    try FileManager.default.copyItem(at: source, to: destination)
                } catch {
                    syncPhase = .error
                    syncMessage = Self.userFacingOperationError(
                        error,
                        fallback: "Export skipped \(attachment.filename)"
                    )
                }
            }
        }
        return written
    }

    func importMarkdownFiles(_ files: [ImportFileDto]) -> Int {
        guard !files.isEmpty else { return 0 }
        let notebookId: String
        if let existing = currentNotebookId() {
            notebookId = existing
        } else if let nb = try? engine.createNotebook(name: "Notes", parentId: nil) {
            notebookId = nb.id
        } else {
            return 0
        }
        let importedIds = (try? engine.importMarkdown(notebookId: notebookId, files: files)) ?? []
        selection = .notebook(notebookId)
        reload()
        selectedNoteId = importedIds.first
        scheduleAutoSync(reason: "Imported notes will back up shortly")
        return importedIds.count
    }

    func registerSync(email: String, password: String) {
        authenticateSync(email: email, password: password, mode: .register)
    }

    func loginSync(email: String, password: String) {
        authenticateSync(email: email, password: password, mode: .login)
    }

    func refreshSyncSession() {
        guard isSyncAuthenticated else {
            syncPhase = .localOnly
            syncMessage = "Sign in to enable sync"
            return
        }

        beginSyncWork("Refreshing session...")
        let engine = self.engine
        let baseURL = normalizedSyncBaseURL()
        let token = syncToken
        Task.detached { [engine] in
            do {
                let session = try engine.refreshHttp(baseUrl: baseURL, token: token)
                await MainActor.run {
                    self.applySyncSession(session, email: self.syncEmail, baseURL: baseURL)
                    self.finishSyncWork("Session refreshed")
                }
            } catch {
                await MainActor.run {
                    self.failSyncWork(error)
                }
            }
        }
    }

    func syncNow() {
        guard isSyncAuthenticated else {
            syncPhase = .localOnly
            syncMessage = "Sign in to enable sync"
            return
        }
        guard backupEncryptionAllowsSync else {
            syncPhase = .error
            syncMessage = syncEncryptionEnabled
                ? "Unlock backup encryption to sync"
                : "Enable backup encryption to sync"
            return
        }

        if isSyncing {
            syncAgainAfterCurrentRun = true
            syncMessage = "Sync queued"
            return
        }

        pendingAutoSyncTask?.cancel()
        pendingAutoSyncTask = nil
        beginSyncWork("Syncing...")
        let engine = self.engine
        let baseURL = normalizedSyncBaseURL()
        let token = syncToken
        let deviceId = syncDeviceId
        let attachmentDir = (try? Self.attachmentDirectory().path) ?? ""
        Task.detached { [engine] in
            do {
                let report = try engine.syncHttpWithBlobs(
                    baseUrl: baseURL,
                    token: token,
                    deviceId: deviceId,
                    attachmentDir: attachmentDir
                )
                await MainActor.run {
                    self.reload()
                    self.finishSyncWork(self.summary(for: report))
                }
            } catch {
                await MainActor.run {
                    self.failSyncWork(error)
                }
            }
        }
    }

    func signOutSync() {
        pendingAutoSyncTask?.cancel()
        pendingAutoSyncTask = nil
        syncAgainAfterCurrentRun = false
        syncToken = ""
        syncUserId = ""
        syncDeviceId = ""
        syncPhase = .localOnly
        syncMessage = "Local notes only"
        saveSyncSettings()
    }

    func useLocalWranglerSyncServer() {
        syncBaseURL = Self.defaultSyncBaseURL
        syncMessage = "Using local Wrangler D1 backup"
        if !isSyncAuthenticated {
            syncPhase = .localOnly
        }
        saveSyncSettings()
    }

    func checkSyncServer() {
        beginSyncWork("Checking server...")
        let baseURL = normalizedSyncBaseURL()
        guard let url = URL(string: "\(baseURL)/health") else {
            isSyncing = false
            syncPhase = .error
            syncMessage = "Invalid server URL"
            return
        }

        Task.detached {
            do {
                var request = URLRequest(url: url)
                request.timeoutInterval = 5
                let (_, response) = try await URLSession.shared.data(for: request)
                guard let http = response as? HTTPURLResponse,
                      (200..<300).contains(http.statusCode) else {
                    throw URLError(.badServerResponse)
                }
                await MainActor.run {
                    self.isSyncing = false
                    self.syncBaseURL = baseURL
                    self.syncPhase = self.isSyncConfigured ? .ready : (self.isSyncAuthenticated ? .error : .localOnly)
                    self.syncMessage = self.isSyncAuthenticated && !self.backupEncryptionAllowsSync
                        ? (self.syncEncryptionEnabled
                            ? "Server reachable. Unlock backup encryption to sync"
                            : "Server reachable. Enable backup encryption to sync")
                        : "Server reachable"
                    self.saveSyncSettings()
                }
            } catch {
                await MainActor.run {
                    self.failSyncWork(error)
                }
            }
        }
    }

    func enableBackupEncryption(passphrase: String) {
        guard !syncEncryptionUnlocked else {
            syncMessage = "Backup encryption is already unlocked"
            syncPhase = isSyncConfigured ? .ready : syncPhase
            return
        }

        let trimmed = passphrase.trimmingCharacters(in: .whitespacesAndNewlines)
        guard trimmed.count >= 8 else {
            syncPhase = .error
            syncMessage = "Backup key must be at least 8 characters"
            return
        }

        do {
            let salt: String
            if syncEncryptionSalt.isEmpty {
                salt = try Self.generateBackupEncryptionSalt()
            } else {
                salt = syncEncryptionSalt
            }
            let encryptedEngine = try KansoEngine.openWithEncryptionPassphrase(
                path: Self.databasePath(),
                passphrase: trimmed,
                salt: salt
            )
            try Self.storeBackupEncryptionPassphrase(trimmed)
            syncEncryptionSalt = salt
            syncEncryptionEnabled = true
            syncEncryptionUnlocked = true
            engine = encryptedEngine
            reload()
            saveSyncSettings()
            syncPhase = isSyncAuthenticated ? .ready : .localOnly
            syncMessage = isSyncAuthenticated ? "Backup encryption unlocked" : "Backup encryption enabled"
            if isSyncAuthenticated {
                scheduleAutoSync(reason: "Encrypted backup will run shortly")
            }
        } catch {
            syncPhase = .error
            syncMessage = Self.userFacingOperationError(error, fallback: "Backup encryption failed")
        }
    }

    func reloadAgentSettings() {
        let clients = (try? engine.listMcpClients()) ?? []
        mcpClients = clients
        var capabilities: [String: Set<String>] = [:]
        for client in clients {
            capabilities[client.id] = Set((try? engine.listMcpCapabilities(clientId: client.id)) ?? [])
        }
        mcpCapabilities = capabilities

        let loadedSkills = (try? engine.listSkills()) ?? []
        skills = loadedSkills
        var runs: [String: [SkillRunDto]] = [:]
        for skill in loadedSkills {
            runs[skill.id] = (try? engine.listSkillRuns(skillId: skill.id)) ?? []
        }
        skillRuns = runs
    }

    func capabilities(for clientId: String) -> Set<String> {
        mcpCapabilities[clientId] ?? []
    }

    func registerMcpClient(name: String) {
        let trimmed = name.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return }
        do {
            _ = try engine.registerMcpClient(name: trimmed)
            settingsMessage = "MCP client registered"
            reloadAgentSettings()
        } catch {
            settingsMessage = Self.userFacingOperationError(error, fallback: "MCP client failed")
        }
    }

    func setMcpClientTrusted(_ clientId: String, trusted: Bool) {
        do {
            try engine.setMcpClientTrusted(clientId: clientId, trusted: trusted)
            reloadAgentSettings()
        } catch {
            settingsMessage = Self.userFacingOperationError(error, fallback: "Trust update failed")
        }
    }

    func setMcpCapability(_ capability: String, for clientId: String, enabled: Bool) {
        do {
            if enabled {
                try engine.grantMcpCapability(clientId: clientId, capability: capability)
            } else {
                try engine.revokeMcpCapability(clientId: clientId, capability: capability)
            }
            reloadAgentSettings()
        } catch {
            settingsMessage = Self.userFacingOperationError(error, fallback: "Capability update failed")
        }
    }

    @discardableResult
    func createSkill(title: String, bodyMarkdown: String, scope: String) -> SkillDto? {
        let trimmedTitle = title.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmedTitle.isEmpty else { return nil }
        do {
            let skill = try engine.createSkill(
                title: trimmedTitle,
                bodyMarkdown: bodyMarkdown,
                scope: scope
            )
            settingsMessage = "Skill created"
            reloadAgentSettings()
            return skill
        } catch {
            settingsMessage = Self.userFacingOperationError(error, fallback: "Skill create failed")
            return nil
        }
    }

    func updateSkill(_ skillId: String, title: String, bodyMarkdown: String, scope: String, enabled: Bool) {
        let trimmedTitle = title.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmedTitle.isEmpty else { return }
        do {
            try engine.updateSkill(
                skillId: skillId,
                title: trimmedTitle,
                bodyMarkdown: bodyMarkdown,
                scope: scope,
                enabled: enabled
            )
            settingsMessage = "Skill saved"
            reloadAgentSettings()
        } catch {
            settingsMessage = Self.userFacingOperationError(error, fallback: "Skill save failed")
        }
    }

    func deleteSkill(_ skillId: String) {
        do {
            try engine.deleteSkill(skillId: skillId)
            settingsMessage = "Skill deleted"
            reloadAgentSettings()
        } catch {
            settingsMessage = Self.userFacingOperationError(error, fallback: "Skill delete failed")
        }
    }

    func runs(for skillId: String) -> [SkillRunDto] {
        skillRuns[skillId] ?? []
    }

    func requestSettings(_ section: String) {
        requestedSettingsSection = section
        isSettingsPresented = true
    }

    func requestEditorAction(_ action: KansoEditorAction) {
        guard selectedNoteId != nil else {
            syncPhase = .error
            syncMessage = "Select a note first"
            return
        }
        pendingEditorAction = action
    }

    func runFirstEnabledSkillOnCurrentNote() {
        guard let noteId = selectedNoteId,
              let skill = skills.first(where: { $0.enabled }) else {
            settingsMessage = "Create an enabled skill first"
            requestSettings("skills")
            return
        }

        do {
            let run = try engine.startSkillRun(
                skillId: skill.id,
                targetType: "note",
                targetId: noteId,
                mode: "review_changes"
            )
            settingsMessage = "Skill run queued: \(skill.title)"
            skillRuns[skill.id, default: []].insert(run, at: 0)
        } catch {
            settingsMessage = Self.userFacingOperationError(error, fallback: "Skill run failed")
        }
    }

    func shareMembers(resourceType: String, resourceId: String) -> [ShareMemberDto] {
        (try? engine.listShareMembers(resourceType: resourceType, resourceId: resourceId)) ?? []
    }

    @discardableResult
    func addShareMember(resourceType: String, resourceId: String, email: String, role: String) -> ShareMemberDto? {
        let trimmedEmail = email.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmedEmail.isEmpty else { return nil }
        do {
            let member = try engine.addShareMember(
                resourceType: resourceType,
                resourceId: resourceId,
                email: trimmedEmail,
                role: role
            )
            syncMessage = "Share invite saved"
            scheduleAutoSync(reason: "Share invite will back up shortly")
            return member
        } catch {
            syncPhase = .error
            syncMessage = Self.userFacingOperationError(error, fallback: "Share failed")
            return nil
        }
    }

    func removeShareMember(_ memberId: String) {
        do {
            try engine.removeShareMember(memberId: memberId)
            syncMessage = "Share member removed"
            scheduleAutoSync(reason: "Share removal will back up shortly")
        } catch {
            syncPhase = .error
            syncMessage = Self.userFacingOperationError(error, fallback: "Remove share failed")
        }
    }

    func saveSyncSettings() {
        let defaults = UserDefaults.standard
        defaults.set(normalizedSyncBaseURL(), forKey: SyncDefaults.baseURL)
        defaults.set(syncEmail, forKey: SyncDefaults.email)
        defaults.set(syncToken, forKey: SyncDefaults.token)
        defaults.set(syncUserId, forKey: SyncDefaults.userId)
        defaults.set(syncDeviceId, forKey: SyncDefaults.deviceId)
        defaults.set(syncEncryptionEnabled, forKey: SyncDefaults.encryptionEnabled)
        defaults.set(syncEncryptionSalt, forKey: SyncDefaults.encryptionSalt)
    }

    func deleteNote(_ noteId: String) {
        try? engine.deleteNote(noteId: noteId)
        reload()
        scheduleAutoSync(reason: "Deleted note will back up shortly")
    }

    func restoreNote(_ noteId: String) {
        try? engine.restoreNote(noteId: noteId)
        selection = .all
        reload()
        selectedNoteId = noteId
        scheduleAutoSync(reason: "Restored note will back up shortly")
    }

    func purgeNote(_ noteId: String) {
        try? engine.purgeNote(noteId: noteId)
        reload()
        scheduleAutoSync(reason: "Purged note will back up shortly")
    }

    func note(_ id: String?) -> NoteDto? {
        guard let id else { return nil }
        return ((try? engine.getNote(noteId: id)) ?? nil)
            ?? trashNotes.first { $0.id == id }
    }

    func isTrashNote(_ noteId: String) -> Bool {
        trashNotes.contains { $0.id == noteId }
    }

    private func replaceNoteIfPresent(_ noteId: String) {
        guard let updated = (try? engine.getNote(noteId: noteId)) ?? nil else {
            reload()
            return
        }
        if let allIndex = allNotes.firstIndex(where: { $0.id == noteId }) {
            allNotes[allIndex] = updated
        }
        if let noteIndex = notes.firstIndex(where: { $0.id == noteId }) {
            notes[noteIndex] = updated
        }
    }

    private func promotePlaceholderTitles(in notes: [NoteDto]) -> Bool {
        var promoted = false
        for note in notes where shouldPromotePlaceholderTitle(note.title) {
            guard let title = Self.derivedTitle(fromMarkdown: note.bodyMarkdown),
                  title != note.title else { continue }
            do {
                try engine.renameNote(noteId: note.id, title: title)
                promoted = true
            } catch {
                syncPhase = .error
                syncMessage = Self.userFacingOperationError(error, fallback: "Title update failed")
            }
        }
        return promoted
    }

    private func shouldPromotePlaceholderTitle(_ title: String) -> Bool {
        let normalized = title.trimmingCharacters(in: .whitespacesAndNewlines).localizedLowercase
        return normalized.isEmpty || normalized == "untitled" || normalized == "untitled note"
    }

    private func notesWithOpenTasks(from source: [NoteDto]) -> [NoteDto] {
        let taskNoteIds = Set(openTaskItems.map(\.noteId))
        return source.filter { taskNoteIds.contains($0.id) }
    }

    private func appendNotebook(_ notebook: NotebookDto, depth: Int, into rows: inout [NotebookOutlineItem]) {
        rows.append(NotebookOutlineItem(notebook: notebook, depth: depth))
        for child in childNotebooks(of: notebook.id) {
            appendNotebook(child, depth: depth + 1, into: &rows)
        }
    }

    private func isDescendant(_ possibleChildId: String, of ancestorId: String) -> Bool {
        var current = notebooks.first { $0.id == possibleChildId }
        while let parentId = current?.parentId {
            if parentId == ancestorId {
                return true
            }
            current = notebooks.first { $0.id == parentId }
        }
        return false
    }

    private enum AuthMode {
        case register
        case login
    }

    private func authenticateSync(email: String, password: String, mode: AuthMode) {
        let trimmedEmail = email.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmedEmail.isEmpty, !password.isEmpty else {
            syncPhase = .error
            syncMessage = "Email and password are required"
            return
        }

        beginSyncWork(mode == .register ? "Creating account..." : "Signing in...")
        let engine = self.engine
        let baseURL = normalizedSyncBaseURL()
        Task.detached { [engine] in
            do {
                let session: AuthSessionDto
                switch mode {
                case .register:
                    session = try engine.registerHttp(baseUrl: baseURL, email: trimmedEmail, password: password)
                case .login:
                    session = try engine.loginHttp(baseUrl: baseURL, email: trimmedEmail, password: password)
                }
                await MainActor.run {
                    self.applySyncSession(session, email: trimmedEmail, baseURL: baseURL)
                    self.finishSyncWork("Signed in")
                    self.syncNow()
                }
            } catch {
                await MainActor.run {
                    self.failSyncWork(error)
                }
            }
        }
    }

    private func scheduleAutoSync(
        reason: String,
        delayNanoseconds: UInt64 = 2_000_000_000
    ) {
        guard isSyncAuthenticated else { return }
        guard backupEncryptionAllowsSync else {
            syncPhase = .error
            syncMessage = syncEncryptionEnabled
                ? "Unlock backup encryption to sync"
                : "Enable backup encryption to sync"
            return
        }

        if isSyncing {
            syncAgainAfterCurrentRun = true
            syncMessage = "Sync queued"
            return
        }

        pendingAutoSyncTask?.cancel()
        syncPhase = .ready
        syncMessage = reason
        pendingAutoSyncTask = Task { @MainActor [weak self] in
            do {
                try await Task.sleep(nanoseconds: delayNanoseconds)
            } catch {
                return
            }

            guard let self, !Task.isCancelled else { return }
            self.pendingAutoSyncTask = nil
            self.syncNow()
        }
    }

    private func beginSyncWork(_ message: String) {
        isSyncing = true
        syncPhase = .syncing
        syncMessage = message
        saveSyncSettings()
    }

    private func finishSyncWork(_ message: String) {
        isSyncing = false
        syncPhase = isSyncConfigured ? .ready : .localOnly
        syncMessage = message
        saveSyncSettings()

        if syncAgainAfterCurrentRun {
            syncAgainAfterCurrentRun = false
            scheduleAutoSync(reason: "Sync queued")
        }
    }

    private func failSyncWork(_ error: Error) {
        isSyncing = false
        syncAgainAfterCurrentRun = false
        syncPhase = .error
        syncMessage = Self.userFacingSyncError(error, baseURL: normalizedSyncBaseURL())
        saveSyncSettings()
    }

    private func applySyncSession(_ session: AuthSessionDto, email: String, baseURL: String) {
        syncBaseURL = baseURL
        syncEmail = email
        syncToken = session.token
        syncUserId = session.userId
        syncDeviceId = session.deviceId
        saveSyncSettings()
    }

    private func summary(for report: SyncReportDto) -> String {
        if report.pushed == 0,
           report.applied == 0,
           report.conflicted == 0,
           report.deleted == 0,
           report.skipped == 0 {
            return "Synced just now"
        }
        var parts: [String] = []
        if report.pushed > 0 { parts.append("\(report.pushed) pushed") }
        if report.applied > 0 { parts.append("\(report.applied) applied") }
        if report.conflicted > 0 { parts.append("\(report.conflicted) conflicted") }
        if report.deleted > 0 { parts.append("\(report.deleted) deleted") }
        if report.skipped > 0 { parts.append("\(report.skipped) skipped") }
        if report.uploadedBlobs > 0 { parts.append("\(report.uploadedBlobs) files uploaded") }
        if report.downloadedBlobs > 0 { parts.append("\(report.downloadedBlobs) files restored") }
        return parts.joined(separator: ", ")
    }

    nonisolated static func userFacingSyncError(_ error: Error, baseURL: String? = nil) -> String {
        let raw = rawEngineMessage(error)
        let normalized = raw.lowercased()

        if normalized.contains("unauthorized")
            || normalized.contains("forbidden")
            || normalized.contains("invalid token")
            || normalized.contains("401")
            || normalized.contains("403") {
            return "Sign in again to continue syncing."
        }

        if normalized.contains("connection refused")
            || normalized.contains("could not connect")
            || normalized.contains("failed to connect")
            || normalized.contains("cannot connect")
            || normalized.contains("timed out")
            || normalized.contains("network connection was lost")
            || normalized.contains("not connected to the internet")
            || normalized.contains("transport error")
            || normalized.contains("nsurlerrordomain")
            || normalized.contains("curl error") {
            if isLocalSyncURL(baseURL) {
                return "Can't reach sync server. Start Wrangler or check the server address."
            }
            return "Can't reach sync server. Check the server address or your connection."
        }

        if normalized.contains("bad server response")
            || normalized.contains("server returned")
            || normalized.contains("status 404")
            || normalized.contains("status 500")
            || normalized.contains("502")
            || normalized.contains("503") {
            return "Sync server is not responding correctly. Check the server address."
        }

        if normalized.contains("encryption")
            || normalized.contains("decrypt")
            || normalized.contains("passphrase")
            || normalized.contains("backup key") {
            return "Check backup encryption before syncing."
        }

        return "Sync failed. Try again from Sync settings."
    }

    nonisolated static func userFacingOperationError(_ error: Error, fallback: String) -> String {
        let raw = rawEngineMessage(error).trimmingCharacters(in: .whitespacesAndNewlines)
        guard !raw.isEmpty else { return fallback }
        return "\(fallback): \(cleanUserFacingDetail(raw))"
    }

    nonisolated private static func rawEngineMessage(_ error: Error) -> String {
        if let kansoError = error as? KansoError {
            switch kansoError {
            case .Engine(let message):
                return message
            }
        }

        if let localized = (error as? LocalizedError)?.errorDescription,
           !localized.isEmpty {
            return localized
        }

        let localizedDescription = error.localizedDescription
        if !localizedDescription.isEmpty {
            return localizedDescription
        }

        return String(describing: error)
    }

    nonisolated private static func cleanUserFacingDetail(_ detail: String) -> String {
        var cleaned = detail
        if let range = cleaned.range(of: #"Engine\(message: "([^"]*)""#, options: .regularExpression) {
            let fragment = String(cleaned[range])
            if let firstQuote = fragment.firstIndex(of: "\""),
               let lastQuote = fragment.lastIndex(of: "\""),
               firstQuote < lastQuote {
                cleaned = String(fragment[fragment.index(after: firstQuote)..<lastQuote])
            }
        }
        cleaned = cleaned.replacingOccurrences(of: #"\""#, with: "\"")
        cleaned = cleaned.replacingOccurrences(of: "\\n", with: " ")
        return cleaned.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    nonisolated private static func isLocalSyncURL(_ baseURL: String?) -> Bool {
        guard let baseURL else { return false }
        let normalized = baseURL.lowercased()
        return normalized.contains("127.0.0.1") || normalized.contains("localhost")
    }

    private func normalizedSyncBaseURL() -> String {
        let trimmed = syncBaseURL.trimmingCharacters(in: .whitespacesAndNewlines)
        return trimmed.isEmpty ? Self.defaultSyncBaseURL : trimmed.trimmingCharacters(in: CharacterSet(charactersIn: "/"))
    }

    private func loadSyncSettings() {
        let defaults = UserDefaults.standard
        let storedBaseURL = defaults.string(forKey: SyncDefaults.baseURL)?
            .trimmingCharacters(in: .whitespacesAndNewlines)
            .trimmingCharacters(in: CharacterSet(charactersIn: "/"))
        if let storedBaseURL,
           !storedBaseURL.isEmpty,
           !Self.legacySyncBaseURLs.contains(storedBaseURL) {
            syncBaseURL = storedBaseURL
        } else {
            syncBaseURL = Self.defaultSyncBaseURL
            defaults.set(syncBaseURL, forKey: SyncDefaults.baseURL)
        }
        syncEmail = defaults.string(forKey: SyncDefaults.email) ?? ""
        syncToken = defaults.string(forKey: SyncDefaults.token) ?? ""
        syncUserId = defaults.string(forKey: SyncDefaults.userId) ?? ""
        syncDeviceId = defaults.string(forKey: SyncDefaults.deviceId) ?? ""
        if isSyncAuthenticated, !backupEncryptionAllowsSync {
            syncPhase = .error
            syncMessage = syncEncryptionEnabled
                ? "Unlock backup encryption to sync"
                : "Enable backup encryption to sync"
        } else {
            syncPhase = isSyncConfigured ? .ready : .localOnly
            syncMessage = isSyncConfigured ? "Ready to sync" : "Local notes only"
        }
    }

    private enum SyncDefaults {
        static let baseURL = "KansoSyncBaseURL"
        static let email = "KansoSyncEmail"
        static let token = "KansoSyncToken"
        static let userId = "KansoSyncUserId"
        static let deviceId = "KansoSyncDeviceId"
        static let encryptionEnabled = "KansoSyncEncryptionEnabled"
        static let encryptionSalt = "KansoSyncEncryptionSalt"
    }

    private static func loadBackupEncryptionSettings() -> (enabled: Bool, salt: String) {
        let defaults = UserDefaults.standard
        return (
            defaults.bool(forKey: SyncDefaults.encryptionEnabled),
            defaults.string(forKey: SyncDefaults.encryptionSalt) ?? ""
        )
    }

    private static func generateBackupEncryptionSalt() throws -> String {
        var bytes = [UInt8](repeating: 0, count: 16)
        let status = bytes.withUnsafeMutableBytes { buffer in
            SecRandomCopyBytes(kSecRandomDefault, buffer.count, buffer.baseAddress!)
        }
        guard status == errSecSuccess else {
            throw keychainError("Could not create backup encryption salt", status: status)
        }
        return Data(bytes).base64EncodedString()
    }

    private static func storeBackupEncryptionPassphrase(_ passphrase: String) throws {
        guard let data = passphrase.data(using: .utf8) else {
            throw keychainError("Could not encode backup encryption key", status: errSecParam)
        }
        var query = backupEncryptionKeychainQuery()
        SecItemDelete(query as CFDictionary)
        query[kSecValueData as String] = data
        query[kSecAttrAccessible as String] = kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly
        let status = SecItemAdd(query as CFDictionary, nil)
        guard status == errSecSuccess else {
            throw keychainError("Could not save backup encryption key", status: status)
        }
    }

    private static func loadBackupEncryptionPassphrase() -> String? {
        var query = backupEncryptionKeychainQuery()
        query[kSecReturnData as String] = kCFBooleanTrue
        query[kSecMatchLimit as String] = kSecMatchLimitOne

        var item: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &item)
        guard status == errSecSuccess,
              let data = item as? Data else {
            return nil
        }
        return String(data: data, encoding: .utf8)
    }

    private static func backupEncryptionKeychainQuery() -> [String: Any] {
        [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: backupEncryptionService,
            kSecAttrAccount as String: backupEncryptionAccount
        ]
    }

    private static func keychainError(_ message: String, status: OSStatus) -> NSError {
        NSError(
            domain: "KansoKeychain",
            code: Int(status),
            userInfo: [NSLocalizedDescriptionKey: "\(message) (\(status))"]
        )
    }

    private static func databasePath() throws -> String {
        if let override = ProcessInfo.processInfo.environment["KANSO_DB_PATH"],
           !override.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            let url = URL(fileURLWithPath: override)
            try FileManager.default.createDirectory(
                at: url.deletingLastPathComponent(),
                withIntermediateDirectories: true
            )
            return url.path
        }

        return try supportDirectory().appendingPathComponent("kanso.db").path
    }

    private static func supportDirectory() throws -> URL {
        let support = FileManager.default
            .urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
        let dir = support.appendingPathComponent("Kanso", isDirectory: true)
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        return dir
    }

    private static func attachmentDirectory() throws -> URL {
        let dir = try supportDirectory().appendingPathComponent("Attachments", isDirectory: true)
        try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        return dir
    }

    private static func copyAttachmentIntoLibrary(_ source: URL) throws -> StoredAttachment {
        let filename = safeFilename(source.lastPathComponent)
        let hash = try sha256Hex(for: source)
        let size = try source.resourceValues(forKeys: [.fileSizeKey]).fileSize.map(Int64.init) ?? 0
        let mimeType = source.pathExtension.isEmpty
            ? "application/octet-stream"
            : (UTType(filenameExtension: source.pathExtension)?.preferredMIMEType ?? "application/octet-stream")
        let directory = try attachmentDirectory().appendingPathComponent(hash, isDirectory: true)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        let destination = directory.appendingPathComponent(filename)
        if !FileManager.default.fileExists(atPath: destination.path) {
            try FileManager.default.copyItem(at: source, to: destination)
        }
        return StoredAttachment(
            filename: filename,
            mimeType: mimeType,
            sizeBytes: size,
            contentHash: hash,
            localURL: destination
        )
    }

    private static func sha256Hex(for url: URL) throws -> String {
        let handle = try FileHandle(forReadingFrom: url)
        defer {
            try? handle.close()
        }

        var hasher = SHA256()
        while autoreleasepool(invoking: {
            let data = try? handle.read(upToCount: 1024 * 1024)
            guard let data, !data.isEmpty else { return false }
            hasher.update(data: data)
            return true
        }) {}

        return hasher.finalize().map { String(format: "%02x", $0) }.joined()
    }

    private static func safeFilename(_ filename: String) -> String {
        let replaced = filename.map { character -> Character in
            switch character {
            case "/", "\\", ":", "\0": return "_"
            default: return character
            }
        }
        let cleaned = String(replaced).trimmingCharacters(in: .whitespacesAndNewlines)
        return cleaned.isEmpty ? "attachment" : cleaned
    }

    nonisolated static func uniqueMarkdownFilename(for title: String, usedFilenames: inout Set<String>) -> String {
        let stem = markdownFilenameStem(for: title)
        var filename = "\(stem).md"
        if usedFilenames.insert(filename).inserted {
            return filename
        }

        var suffix = 2
        while true {
            filename = "\(stem) \(suffix).md"
            if usedFilenames.insert(filename).inserted {
                return filename
            }
            suffix += 1
        }
    }

    nonisolated private static func markdownFilenameStem(for title: String) -> String {
        let cleaned = title.map { character -> Character in
            if character.isLetter || character.isNumber || character == " " || character == "-" || character == "_" {
                return character
            }
            return "_"
        }
        let stem = String(cleaned).trimmingCharacters(in: .whitespacesAndNewlines)
        return stem.isEmpty ? "untitled" : stem
    }

    private static func exportBody(_ body: String, attachments: [AttachmentDto]) -> String {
        var exported = body
        for attachment in attachments {
            let exportPath = exportPath(for: attachment)
            let linkPath = markdownLinkPath(exportPath)
            let replacement = attachment.mimeType.hasPrefix("image/")
                ? "![\(attachment.filename)](\(linkPath))"
                : "[\(attachment.filename)](\(linkPath))"
            let suffix = attachment.id.hasPrefix("attachment:")
                ? String(attachment.id.dropFirst("attachment:".count))
                : attachment.id
            let refs = [
                "![[attachment:\(suffix)]]",
                "![[\(attachment.id)]]",
                "![[attachment:\(attachment.filename)]]"
            ]
            for ref in refs {
                exported = exported.replacingOccurrences(of: ref, with: replacement)
            }
        }
        return exported
    }

    private static func exportPath(for attachment: AttachmentDto) -> String {
        "attachments/\(attachment.contentHash)/\(safeFilename(attachment.filename))"
    }

    private static func markdownLinkPath(_ path: String) -> String {
        path
            .replacingOccurrences(of: " ", with: "%20")
            .replacingOccurrences(of: "(", with: "%28")
            .replacingOccurrences(of: ")", with: "%29")
    }

    private static func effectiveTitle(requestedTitle: String, bodyMarkdown: String) -> String {
        let trimmed = requestedTitle.trimmingCharacters(in: .whitespacesAndNewlines)
        let isPlaceholder = trimmed.isEmpty
            || trimmed.localizedLowercase == "untitled"
            || trimmed.localizedLowercase == "untitled note"
        if isPlaceholder, let title = derivedTitle(fromMarkdown: bodyMarkdown) {
            return title
        }
        return trimmed.isEmpty ? "Untitled" : trimmed
    }

    private static func canonicalWikiTarget(_ raw: String) -> String? {
        var target = raw
            .removingPercentEncoding?
            .trimmingCharacters(in: .whitespacesAndNewlines)
            ?? raw.trimmingCharacters(in: .whitespacesAndNewlines)
        if let pipe = target.firstIndex(of: "|") {
            target = String(target[..<pipe]).trimmingCharacters(in: .whitespacesAndNewlines)
        }
        if let heading = target.firstIndex(of: "#") {
            target = String(target[..<heading]).trimmingCharacters(in: .whitespacesAndNewlines)
        }
        if target.hasPrefix("note:") {
            return target
        }
        return target.isEmpty ? nil : target
    }

    private static func derivedTitle(fromMarkdown body: String) -> String? {
        var skippingFrontMatter = false
        var firstLine = true

        for rawLine in body.components(separatedBy: .newlines) {
            let trimmed = rawLine.trimmingCharacters(in: .whitespacesAndNewlines)
            if firstLine, trimmed == "---" {
                firstLine = false
                skippingFrontMatter = true
                continue
            }
            firstLine = false

            if skippingFrontMatter {
                if trimmed == "---" {
                    skippingFrontMatter = false
                }
                continue
            }

            guard !trimmed.isEmpty else { continue }
            guard let title = cleanTitleCandidate(trimmed) else { continue }
            return title
        }

        return nil
    }

    private static func cleanTitleCandidate(_ raw: String) -> String? {
        var title = raw.trimmingCharacters(in: .whitespacesAndNewlines)
        while title.hasPrefix(">") {
            title.removeFirst()
            title = title.trimmingCharacters(in: .whitespacesAndNewlines)
        }
        if let range = title.range(of: #"^#{1,6}\s+"#, options: .regularExpression) {
            title.removeSubrange(range)
        } else if title.first == "#" {
            title = String(title.drop(while: { $0 == "#" }))
                .trimmingCharacters(in: .whitespacesAndNewlines)
        }
        if let range = title.range(of: #"^([-*+]|\d+[.)])\s+"#, options: .regularExpression) {
            title.removeSubrange(range)
        }
        if let range = title.range(of: #"^\[[ xX]\]\s+"#, options: .regularExpression) {
            title.removeSubrange(range)
        }
        if let range = title.range(of: #"\s+#+$"#, options: .regularExpression) {
            title.removeSubrange(range)
        }
        title = title
            .replacingOccurrences(of: "**", with: "")
            .replacingOccurrences(of: "__", with: "")
            .replacingOccurrences(of: "`", with: "")
            .trimmingCharacters(in: .whitespacesAndNewlines)

        guard !title.isEmpty else { return nil }
        if title.count > 80 {
            title = String(title.prefix(80)).trimmingCharacters(in: .whitespacesAndNewlines)
        }
        return title
    }

    private struct StoredAttachment {
        let filename: String
        let mimeType: String
        let sizeBytes: Int64
        let contentHash: String
        let localURL: URL
    }
}

private extension String {
    func rstrip() -> String {
        var copy = self
        while copy.last?.isWhitespace == true {
            copy.removeLast()
        }
        return copy
    }

    func removingAttachmentReferences(for attachment: AttachmentDto) -> String {
        let suffix = attachment.id.hasPrefix("attachment:")
            ? String(attachment.id.dropFirst("attachment:".count))
            : attachment.id
        let refs = [
            "![[attachment:\(suffix)]]",
            "![[\(attachment.id)]]",
            "![[attachment:\(attachment.filename)]]"
        ]

        var next = self
        for ref in refs {
            next = next.replacingOccurrences(of: ref, with: "")
        }

        while next.contains("\n\n\n") {
            next = next.replacingOccurrences(of: "\n\n\n", with: "\n\n")
        }
        return next.rstrip()
    }
}
