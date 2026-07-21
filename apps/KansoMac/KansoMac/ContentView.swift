import SwiftUI
import AppKit
import UniformTypeIdentifiers

/// Three-pane shell: notebook source list, note list, and a Markdown workspace.
struct ContentView: View {
    @EnvironmentObject var store: KansoStore

    var body: some View {
        NavigationSplitView {
            SidebarView(showingSettings: $store.isSettingsPresented)
                .navigationSplitViewColumnWidth(min: 200, ideal: 220, max: 260)
        } content: {
            NoteListView()
                .navigationSplitViewColumnWidth(min: 280, ideal: 320, max: 420)
        } detail: {
            EditorView()
        }
        .navigationSplitViewStyle(.balanced)
        .overlay {
            if store.isCommandPalettePresented {
                CommandPaletteView()
                    .environmentObject(store)
            }
        }
        .sheet(isPresented: $store.isSettingsPresented) {
            SettingsView()
                .environmentObject(store)
        }
    }
}

// MARK: - Sidebar

private struct SidebarView: View {
    @EnvironmentObject var store: KansoStore
    @Binding var showingSettings: Bool
    @State private var showingNewNotebook = false
    @State private var newNotebookName = ""
    @State private var renameNotebookId: String?
    @State private var renameNotebookName = ""
    @State private var childNotebookParentId: String?
    @State private var childNotebookName = ""
    @State private var deletingNotebook: NotebookDto?
    @State private var sharingNotebookId: String?

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
                    SidebarRow(icon: "checklist", label: "Tasks", count: store.openTaskItems.count,
                               selected: store.selection == .tasks) { store.select(.tasks) }

                    notebooksHeader
                    ForEach(store.notebookOutline) { item in
                        notebookRow(item)
                    }

                    if !store.tags.isEmpty {
                        sectionLabel("TAGS")
                        ForEach(store.tags, id: \.id) { tag in
                            SidebarRow(icon: "tag", label: tag.name,
                                       count: store.tagCount(tag.id),
                                       selected: store.selection == .tag(tag.id)) {
                                store.select(.tag(tag.id))
                            }
                        }
                    }

                    SidebarRow(icon: "trash", label: "Trash", count: store.trashCount,
                               selected: store.selection == .trash) { store.select(.trash) }
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
                    Text(store.syncStatusLine).font(.system(size: 10)).foregroundStyle(syncStatusColor)
                }
                Spacer()
                Button { showingSettings = true } label: {
                    Image(systemName: "gearshape").font(.system(size: 12)).foregroundStyle(Theme.textMuted)
                }
                .buttonStyle(.plain)
                .help("Settings")
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
        .alert("Rename Notebook", isPresented: renameNotebookBinding) {
            TextField("Name", text: $renameNotebookName)
            Button("Rename") {
                if let id = renameNotebookId {
                    store.renameNotebook(id, name: renameNotebookName)
                }
                renameNotebookId = nil
                renameNotebookName = ""
            }
            Button("Cancel", role: .cancel) {
                renameNotebookId = nil
                renameNotebookName = ""
            }
        } message: {
            Text("Update the notebook name.")
        }
        .alert("New Child Notebook", isPresented: childNotebookBinding) {
            TextField("Name", text: $childNotebookName)
            Button("Create") {
                let name = childNotebookName.trimmingCharacters(in: .whitespaces)
                if !name.isEmpty {
                    store.createNotebook(name: name, parentId: childNotebookParentId)
                }
                childNotebookParentId = nil
                childNotebookName = ""
            }
            Button("Cancel", role: .cancel) {
                childNotebookParentId = nil
                childNotebookName = ""
            }
        } message: {
            Text("Create it inside \(childNotebookParentId.map(store.notebookName) ?? "Notebook").")
        }
        .alert("Delete Notebook?", isPresented: deleteNotebookBinding) {
            Button("Delete", role: .destructive) {
                if let notebook = deletingNotebook {
                    _ = store.deleteNotebook(notebook.id)
                }
                deletingNotebook = nil
            }
            Button("Cancel", role: .cancel) { deletingNotebook = nil }
        } message: {
            Text("Only empty notebooks can be deleted.")
        }
        .sheet(isPresented: shareNotebookBinding) {
            if let id = sharingNotebookId {
                ShareMembersSheet(
                    resourceType: "notebook",
                    resourceId: id,
                    title: store.notebookName(id)
                )
                .environmentObject(store)
            }
        }
    }

    private func notebookRow(_ item: NotebookOutlineItem) -> some View {
        let notebook = item.notebook
        return SidebarRow(
            icon: store.childNotebooks(of: notebook.id).isEmpty ? "book.closed" : "books.vertical",
            label: notebook.name,
            count: store.recursiveNoteCount(forNotebook: notebook.id),
            selected: store.selection == .notebook(notebook.id),
            indent: CGFloat(item.depth) * 12
        ) {
            store.select(.notebook(notebook.id))
        }
        .contextMenu {
            Button {
                childNotebookParentId = notebook.id
                childNotebookName = ""
            } label: {
                Label("New Child Notebook", systemImage: "plus")
            }
            Button {
                renameNotebookId = notebook.id
                renameNotebookName = notebook.name
            } label: {
                Label("Rename", systemImage: "pencil")
            }
            Button {
                sharingNotebookId = notebook.id
            } label: {
                Label("Share", systemImage: "person.2")
            }
            Menu {
                Button {
                    store.moveNotebook(notebook.id, parentId: nil)
                } label: {
                    Label("Root", systemImage: "sidebar.left")
                }
                .disabled(notebook.parentId == nil)

                ForEach(store.notebooks.filter { $0.id != notebook.id }, id: \.id) { candidate in
                    Button(candidate.name) {
                        store.moveNotebook(notebook.id, parentId: candidate.id)
                    }
                    .disabled(!store.canMoveNotebook(notebook.id, under: candidate.id))
                }
            } label: {
                Label("Move To", systemImage: "folder")
            }
            Divider()
            Button(role: .destructive) {
                deletingNotebook = notebook
            } label: {
                Label("Delete", systemImage: "trash")
            }
        }
    }

    private var renameNotebookBinding: Binding<Bool> {
        Binding {
            renameNotebookId != nil
        } set: { isPresented in
            if !isPresented {
                renameNotebookId = nil
                renameNotebookName = ""
            }
        }
    }

    private var childNotebookBinding: Binding<Bool> {
        Binding {
            childNotebookParentId != nil
        } set: { isPresented in
            if !isPresented {
                childNotebookParentId = nil
                childNotebookName = ""
            }
        }
    }

    private var deleteNotebookBinding: Binding<Bool> {
        Binding {
            deletingNotebook != nil
        } set: { isPresented in
            if !isPresented {
                deletingNotebook = nil
            }
        }
    }

    private var shareNotebookBinding: Binding<Bool> {
        Binding {
            sharingNotebookId != nil
        } set: { isPresented in
            if !isPresented {
                sharingNotebookId = nil
            }
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

    private var syncStatusColor: Color {
        switch store.syncPhase {
        case .ready: return Theme.success
        case .syncing: return Theme.accent
        case .error: return Theme.warning
        case .localOnly: return Theme.textMuted
        }
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

// MARK: - Settings

private enum SettingsSection: String, CaseIterable, Identifiable {
    case sync
    case editor
    case mcp
    case skills
    case privacy

    var id: String { rawValue }

    var title: String {
        switch self {
        case .sync: return "Sync"
        case .editor: return "Editor"
        case .mcp: return "MCP Access"
        case .skills: return "Skills"
        case .privacy: return "Privacy"
        }
    }

    var icon: String {
        switch self {
        case .sync: return "arrow.triangle.2.circlepath"
        case .editor: return "keyboard"
        case .mcp: return "point.3.connected.trianglepath.dotted"
        case .skills: return "wand.and.stars"
        case .privacy: return "lock"
        }
    }
}

private struct SettingsView: View {
    @EnvironmentObject var store: KansoStore
    @Environment(\.dismiss) private var dismiss
    @State private var selectedSection: SettingsSection = .sync
    @State private var password = ""
    @State private var encryptionPassphrase = ""
    @State private var newMcpClientName = ""
    @State private var selectedSkillId: String?
    @State private var skillTitle = ""
    @State private var skillBody = ""
    @State private var skillScope = "global"
    @State private var skillEnabled = true

    private let skillScopes = ["global", "notebook", "note", "project"]

    var body: some View {
        HStack(spacing: 0) {
            settingsSidebar
            Divider().overlay(Theme.divider)
            settingsDetail
        }
        .frame(width: 760, height: 520)
        .background(Theme.editor)
        .onAppear {
            applyRequestedSettingsSection()
            store.reloadAgentSettings()
            loadFirstSkillIfNeeded()
        }
        .onChange(of: store.requestedSettingsSection) { _, _ in
            applyRequestedSettingsSection()
        }
    }

    private var settingsSidebar: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("Settings")
                .font(.system(size: 18, weight: .semibold))
                .foregroundStyle(Theme.textPrimary)
                .padding(.bottom, 16)
            ForEach(SettingsSection.allCases) { section in
                SettingsRow(icon: section.icon, label: section.title, selected: selectedSection == section) {
                    selectedSection = section
                    if section == .skills {
                        loadFirstSkillIfNeeded()
                    }
                }
            }
            Spacer()
            Button("Done") { dismiss() }
                .buttonStyle(.borderless)
                .foregroundStyle(Theme.textSecondary)
        }
        .padding(20)
        .frame(width: 220)
        .frame(maxHeight: .infinity, alignment: .topLeading)
        .background(Theme.sidebar)
    }

    @ViewBuilder
    private var settingsDetail: some View {
        switch selectedSection {
        case .sync:
            syncSettings
        case .editor:
            editorSettings
        case .mcp:
            mcpSettings
        case .skills:
            skillsSettings
        case .privacy:
            privacySettings
        }
    }

    private var syncSettings: some View {
        VStack(alignment: .leading, spacing: 18) {
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Sync")
                        .font(.system(size: 24, weight: .semibold))
                        .foregroundStyle(Theme.textPrimary)
                    Text(store.syncSettingsDetailLine)
                        .font(.system(size: 12))
                        .foregroundStyle(Theme.textMuted)
                }
                Spacer()
                syncStatusBadge
            }

            settingsGroup {
                SettingsTextField(label: "Server", text: $store.syncBaseURL, prompt: KansoStore.defaultSyncBaseURL)
                SettingsTextField(label: "Email", text: $store.syncEmail, prompt: "you@example.com")
                HStack {
                    Text("Password")
                        .font(.system(size: 12))
                        .foregroundStyle(Theme.textSecondary)
                        .frame(width: 92, alignment: .leading)
                    SecureField("Required for register/login", text: $password)
                        .textFieldStyle(.plain)
                        .font(.system(size: 13))
                        .foregroundStyle(Theme.textPrimary)
                        .padding(.horizontal, 10)
                        .padding(.vertical, 8)
                        .background(RoundedRectangle(cornerRadius: 7).fill(Theme.elevated))
                }
                HStack {
                    Text("Backup Key")
                        .font(.system(size: 12))
                        .foregroundStyle(Theme.textSecondary)
                        .frame(width: 92, alignment: .leading)
                    SecureField(
                        store.syncEncryptionEnabled ? "Required to unlock backup" : "Separate backup encryption key",
                        text: $encryptionPassphrase
                    )
                    .textFieldStyle(.plain)
                    .font(.system(size: 13))
                    .foregroundStyle(Theme.textPrimary)
                    .padding(.horizontal, 10)
                    .padding(.vertical, 8)
                    .background(RoundedRectangle(cornerRadius: 7).fill(Theme.elevated))
                }
            }

            settingsGroup {
                readOnlyRow("User", store.syncUserId.isEmpty ? "Not signed in" : store.syncUserId)
                readOnlyRow("Device", store.syncDeviceId.isEmpty ? "No device session" : store.syncDeviceId)
                readOnlyRow("Backup Encryption", store.backupEncryptionStatus)
                readOnlyRow(
                    "Automatic",
                    store.isSyncConfigured ? "Backs up shortly after edits and on launch" : "Requires sign-in and backup encryption"
                )
            }

            HStack(spacing: 10) {
                Button {
                    store.useLocalWranglerSyncServer()
                } label: {
                    Label("Use Local Wrangler", systemImage: "cloud")
                }
                .disabled(store.isSyncing)

                Button {
                    store.checkSyncServer()
                } label: {
                    Label("Check Server", systemImage: "checkmark.seal")
                }
                .disabled(store.isSyncing)

                Button {
                    store.loginSync(email: store.syncEmail, password: password)
                } label: {
                    Label("Log In", systemImage: "person.crop.circle.badge.checkmark")
                }
                .disabled(store.isSyncing)

                Button {
                    store.registerSync(email: store.syncEmail, password: password)
                } label: {
                    Label("Create Account", systemImage: "person.badge.plus")
                }
                .disabled(store.isSyncing)

                Button {
                    store.enableBackupEncryption(passphrase: encryptionPassphrase)
                    encryptionPassphrase = ""
                } label: {
                    Label(
                        store.backupEncryptionActionTitle,
                        systemImage: "lock.shield"
                    )
                }
                .disabled(store.isSyncing || store.syncEncryptionUnlocked)

                Button {
                    store.syncNow()
                } label: {
                    Label("Sync Now", systemImage: "arrow.triangle.2.circlepath")
                }
                .disabled(store.isSyncing || !store.isSyncConfigured)

                Spacer()

                Button(role: .destructive) {
                    password = ""
                    store.signOutSync()
                } label: {
                    Label("Sign Out", systemImage: "rectangle.portrait.and.arrow.right")
                }
                .disabled(store.isSyncing || !store.isSyncAuthenticated)
            }
            .buttonStyle(.borderless)
            .foregroundStyle(Theme.textSecondary)

            Spacer()
        }
        .padding(26)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .onDisappear { store.saveSyncSettings() }
    }

    private var editorSettings: some View {
        VStack(alignment: .leading, spacing: 18) {
            settingsHeader("Editor", "Choose how Markdown opens and confirm the native editing surface.")
            settingsGroup {
                HStack {
                    Text("Mode")
                        .font(.system(size: 12))
                        .foregroundStyle(Theme.textSecondary)
                        .frame(width: 92, alignment: .leading)
                    EditorModeControl(selection: $store.editorMode)
                    Spacer()
                }
                readOnlyRow("Markdown", "CommonMark + GFM + Kanso references")
                readOnlyRow("Preview", "Native rendered split/preview modes")
            }
            settingsGroup {
                readOnlyRow("Shortcuts", "⌘⌥1 Edit · ⌘⌥2 Preview · ⌘⌥3 Split · ⌘K Quick Open")
                readOnlyRow("Portable", "Notebook export writes Markdown files and attachment folders")
            }
            Spacer()
        }
        .padding(26)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }

    private var mcpSettings: some View {
        VStack(alignment: .leading, spacing: 16) {
            settingsHeader("MCP Access", "Approve agent clients and grant capabilities per client.")
            settingsGroup {
                HStack(spacing: 10) {
                    SettingsTextField(label: "Client", text: $newMcpClientName, prompt: "Claude Desktop")
                    Button {
                        store.registerMcpClient(name: newMcpClientName)
                        newMcpClientName = ""
                    } label: {
                        Label("Register", systemImage: "plus")
                    }
                    .buttonStyle(.borderless)
                    .foregroundStyle(Theme.textPrimary)
                }
            }

            if store.mcpClients.isEmpty {
                EmptySettingsState(icon: "point.3.connected.trianglepath.dotted", title: "No MCP clients")
            } else {
                ScrollView {
                    VStack(spacing: 10) {
                        ForEach(store.mcpClients, id: \.id) { client in
                            McpClientSettingsCard(client: client)
                                .environmentObject(store)
                        }
                    }
                }
            }

            if !store.settingsMessage.isEmpty {
                Text(store.settingsMessage)
                    .font(.system(size: 11))
                    .foregroundStyle(Theme.textMuted)
            }
        }
        .padding(26)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }

    private var skillsSettings: some View {
        VStack(alignment: .leading, spacing: 16) {
            settingsHeader("Skills", "Create local Markdown-defined behaviors agents can run with review.")

            HStack(spacing: 0) {
                VStack(alignment: .leading, spacing: 8) {
                    Button {
                        newSkillDraft()
                    } label: {
                        Label("New Skill", systemImage: "plus")
                    }
                    .buttonStyle(.borderless)
                    .foregroundStyle(Theme.textPrimary)

                    if store.skills.isEmpty {
                        EmptySettingsState(icon: "wand.and.stars", title: "No skills")
                    } else {
                        ScrollView {
                            VStack(spacing: 4) {
                                ForEach(store.skills, id: \.id) { skill in
                                    Button {
                                        selectSkill(skill)
                                    } label: {
                                        VStack(alignment: .leading, spacing: 3) {
                                            HStack(spacing: 6) {
                                                Image(systemName: skill.enabled ? "checkmark.circle.fill" : "circle")
                                                    .font(.system(size: 10))
                                                    .foregroundStyle(skill.enabled ? Theme.success : Theme.textMuted)
                                                Text(skill.title)
                                                    .font(.system(size: 12, weight: .semibold))
                                                    .foregroundStyle(Theme.textPrimary)
                                                    .lineLimit(1)
                                            }
                                            Text(skill.scope)
                                                .font(.system(size: 10))
                                                .foregroundStyle(Theme.textMuted)
                                        }
                                        .frame(maxWidth: .infinity, alignment: .leading)
                                        .padding(8)
                                        .background(RoundedRectangle(cornerRadius: 7)
                                            .fill(selectedSkillId == skill.id ? Theme.elevated : Color.clear))
                                    }
                                    .buttonStyle(.plain)
                                }
                            }
                        }
                    }
                }
                .padding(12)
                .frame(width: 210)
                .frame(maxHeight: .infinity, alignment: .topLeading)
                .background(RoundedRectangle(cornerRadius: 8).fill(Theme.noteList))

                VStack(alignment: .leading, spacing: 10) {
                    SettingsTextField(label: "Title", text: $skillTitle, prompt: "Summarize selected note")
                    HStack {
                        Text("Scope")
                            .font(.system(size: 12))
                            .foregroundStyle(Theme.textSecondary)
                            .frame(width: 92, alignment: .leading)
                        Picker("Scope", selection: $skillScope) {
                            ForEach(skillScopes, id: \.self) { scope in
                                Text(scope).tag(scope)
                            }
                        }
                        .labelsHidden()
                        .frame(width: 170)
                        Toggle("Enabled", isOn: $skillEnabled)
                            .toggleStyle(.checkbox)
                            .foregroundStyle(Theme.textSecondary)
                    }
                    TextEditor(text: $skillBody)
                        .font(.system(size: 13, design: .monospaced))
                        .foregroundStyle(Theme.textPrimary)
                        .scrollContentBackground(.hidden)
                        .background(Theme.elevated)
                        .clipShape(RoundedRectangle(cornerRadius: 8))
                        .frame(minHeight: 190)

                    HStack(spacing: 10) {
                        Button {
                            saveSkill()
                        } label: {
                            Label(selectedSkillId == nil ? "Create" : "Save", systemImage: "checkmark")
                        }
                        .buttonStyle(.borderless)
                        .foregroundStyle(Theme.textPrimary)

                        Button(role: .destructive) {
                            deleteSelectedSkill()
                        } label: {
                            Label("Delete", systemImage: "trash")
                        }
                        .buttonStyle(.borderless)
                        .foregroundStyle(selectedSkillId == nil ? Theme.textMuted : Theme.destructive)
                        .disabled(selectedSkillId == nil)

                        Spacer()
                    }

                    if let selectedSkillId, !store.runs(for: selectedSkillId).isEmpty {
                        Text("Recent Runs")
                            .font(.system(size: 11, weight: .semibold))
                            .foregroundStyle(Theme.textSecondary)
                        ForEach(store.runs(for: selectedSkillId).prefix(3), id: \.id) { run in
                            readOnlyRow(run.status, run.outputSummary ?? run.mode)
                        }
                    }
                }
                .padding(.leading, 14)
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
            }

            if !store.settingsMessage.isEmpty {
                Text(store.settingsMessage)
                    .font(.system(size: 11))
                    .foregroundStyle(Theme.textMuted)
            }
        }
        .padding(26)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }

    private var privacySettings: some View {
        VStack(alignment: .leading, spacing: 18) {
            settingsHeader("Privacy", "Local-first storage with explicit sync and agent access controls.")
            settingsGroup {
                readOnlyRow("Database", "Local SQLite in Application Support")
                readOnlyRow(
                    "Sync",
                    store.isSyncAuthenticated
                        ? (store.isSyncConfigured
                            ? "Encrypted HTTP sync"
                            : "Signed in, backup encryption required")
                        : "Disabled"
                )
                readOnlyRow("Agents", "\(store.mcpClients.count) approved client\(store.mcpClients.count == 1 ? "" : "s")")
            }
            settingsGroup {
                readOnlyRow("Attachments", "Copied into Kanso storage before sync or export")
                readOnlyRow("History", "Revisions and conflict copies are retained")
            }
            Spacer()
        }
        .padding(26)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }

    private var syncStatusBadge: some View {
        HStack(spacing: 7) {
            Circle().fill(syncColor).frame(width: 8, height: 8)
            Text(store.syncPhase.title)
                .font(.system(size: 12, weight: .medium))
                .foregroundStyle(Theme.textSecondary)
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 6)
        .background(RoundedRectangle(cornerRadius: 7).fill(Theme.elevated))
    }

    private var syncColor: Color {
        switch store.syncPhase {
        case .ready: return Theme.success
        case .syncing: return Theme.accent
        case .error: return Theme.warning
        case .localOnly: return Theme.textMuted
        }
    }

    private func settingsHeader(_ title: String, _ subtitle: String) -> some View {
        HStack {
            VStack(alignment: .leading, spacing: 4) {
                Text(title)
                    .font(.system(size: 24, weight: .semibold))
                    .foregroundStyle(Theme.textPrimary)
                Text(subtitle)
                    .font(.system(size: 12))
                    .foregroundStyle(Theme.textMuted)
            }
            Spacer()
        }
    }

    private func settingsGroup<Content: View>(@ViewBuilder content: () -> Content) -> some View {
        VStack(spacing: 10) {
            content()
        }
        .padding(14)
        .background(RoundedRectangle(cornerRadius: 8).fill(Theme.noteList))
    }

    private func readOnlyRow(_ label: String, _ value: String) -> some View {
        HStack {
            Text(label)
                .font(.system(size: 12))
                .foregroundStyle(Theme.textSecondary)
                .frame(width: 92, alignment: .leading)
            Text(value)
                .font(.system(size: 12, design: .monospaced))
                .foregroundStyle(Theme.textMuted)
                .lineLimit(1)
                .truncationMode(.middle)
            Spacer()
        }
    }

    private func selectSkill(_ skill: SkillDto) {
        selectedSkillId = skill.id
        skillTitle = skill.title
        skillBody = skill.bodyMarkdown
        skillScope = skill.scope
        skillEnabled = skill.enabled
    }

    private func newSkillDraft() {
        selectedSkillId = nil
        skillTitle = ""
        skillBody = ""
        skillScope = "global"
        skillEnabled = true
    }

    private func loadFirstSkillIfNeeded() {
        if let selectedSkillId,
           let selected = store.skills.first(where: { $0.id == selectedSkillId }) {
            selectSkill(selected)
        } else if let first = store.skills.first {
            selectSkill(first)
        } else {
            newSkillDraft()
        }
    }

    private func saveSkill() {
        if let selectedSkillId {
            store.updateSkill(
                selectedSkillId,
                title: skillTitle,
                bodyMarkdown: skillBody,
                scope: skillScope,
                enabled: skillEnabled
            )
            loadFirstSkillIfNeeded()
        } else if let skill = store.createSkill(
            title: skillTitle,
            bodyMarkdown: skillBody,
            scope: skillScope
        ) {
            selectSkill(skill)
        }
    }

    private func deleteSelectedSkill() {
        guard let selectedSkillId else { return }
        store.deleteSkill(selectedSkillId)
        newSkillDraft()
        loadFirstSkillIfNeeded()
    }

    private func applyRequestedSettingsSection() {
        guard let raw = store.requestedSettingsSection,
              let section = SettingsSection(rawValue: raw) else {
            return
        }
        selectedSection = section
        store.requestedSettingsSection = nil
    }
}

private struct SettingsRow: View {
    let icon: String
    let label: String
    let selected: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 9) {
                Image(systemName: icon)
                    .font(.system(size: 12))
                    .foregroundStyle(selected ? Theme.accent : Theme.textMuted)
                    .frame(width: 18)
                Text(label)
                    .font(.system(size: 13))
                    .foregroundStyle(selected ? Theme.textPrimary : Theme.textSecondary)
                Spacer()
            }
            .padding(.horizontal, 9)
            .padding(.vertical, 7)
            .background(RoundedRectangle(cornerRadius: 7).fill(selected ? Theme.accent.opacity(0.18) : Color.clear))
        }
        .buttonStyle(.plain)
    }
}

private struct EmptySettingsState: View {
    let icon: String
    let title: String

    var body: some View {
        VStack(spacing: 8) {
            Image(systemName: icon)
                .font(.system(size: 22))
                .foregroundStyle(Theme.textMuted)
            Text(title)
                .font(.system(size: 12, weight: .medium))
                .foregroundStyle(Theme.textMuted)
        }
        .frame(maxWidth: .infinity, minHeight: 96)
    }
}

private struct McpClientSettingsCard: View {
    @EnvironmentObject var store: KansoStore
    let client: McpClientDto

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack(spacing: 10) {
                Image(systemName: client.trusted ? "checkmark.shield.fill" : "app.connected.to.app.below.fill")
                    .font(.system(size: 14))
                    .foregroundStyle(client.trusted ? Theme.success : Theme.accent)
                    .frame(width: 18)
                VStack(alignment: .leading, spacing: 2) {
                    Text(client.name)
                        .font(.system(size: 13, weight: .semibold))
                        .foregroundStyle(Theme.textPrimary)
                        .lineLimit(1)
                    Text(client.id)
                        .font(.system(size: 10, design: .monospaced))
                        .foregroundStyle(Theme.textMuted)
                        .lineLimit(1)
                        .truncationMode(.middle)
                }
                Spacer()
                Toggle("Trusted", isOn: Binding(
                    get: { client.trusted },
                    set: { store.setMcpClientTrusted(client.id, trusted: $0) }
                ))
                .toggleStyle(.checkbox)
                .font(.system(size: 12))
                .foregroundStyle(Theme.textSecondary)
            }

            LazyVGrid(columns: [GridItem(.adaptive(minimum: 112), spacing: 8)], alignment: .leading, spacing: 8) {
                ForEach(store.mcpCapabilityOptions, id: \.self) { capability in
                    Toggle(capabilityTitle(capability), isOn: Binding(
                        get: {
                            client.trusted || store.capabilities(for: client.id).contains(capability)
                        },
                        set: {
                            store.setMcpCapability(capability, for: client.id, enabled: $0)
                        }
                    ))
                    .toggleStyle(.checkbox)
                    .font(.system(size: 11))
                    .foregroundStyle(client.trusted ? Theme.textMuted : Theme.textSecondary)
                    .disabled(client.trusted)
                }
            }
        }
        .padding(12)
        .background(RoundedRectangle(cornerRadius: 8).fill(Theme.noteList))
    }

    private func capabilityTitle(_ capability: String) -> String {
        switch capability {
        case "run_skill": return "Run skills"
        default: return capability.capitalized
        }
    }
}

private struct SettingsTextField: View {
    let label: String
    @Binding var text: String
    let prompt: String

    var body: some View {
        HStack {
            Text(label)
                .font(.system(size: 12))
                .foregroundStyle(Theme.textSecondary)
                .frame(width: 92, alignment: .leading)
            TextField(prompt, text: $text)
                .textFieldStyle(.plain)
                .font(.system(size: 13))
                .foregroundStyle(Theme.textPrimary)
                .padding(.horizontal, 10)
                .padding(.vertical, 8)
                .background(RoundedRectangle(cornerRadius: 7).fill(Theme.elevated))
        }
    }
}

private struct SidebarRow: View {
    let icon: String
    let label: String
    let count: Int?
    let selected: Bool
    var indent: CGFloat = 0
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
            .padding(.leading, indent)
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
                Button { store.createDailyNote() } label: {
                    Image(systemName: "calendar.badge.plus")
                        .font(.system(size: 13))
                        .foregroundStyle(Theme.textSecondary)
                }
                .buttonStyle(.plain)
                .help("Daily note")
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
                    Image(systemName: emptyStateIcon).font(.system(size: 20)).foregroundStyle(Theme.textMuted)
                    Text(emptyStateTitle).font(.system(size: 12)).foregroundStyle(Theme.textMuted)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ScrollView {
                    LazyVStack(spacing: 0) {
                        ForEach(store.notes, id: \.id) { note in
                            NoteRow(
                                note: note,
                                selected: note.id == store.selectedNoteId,
                                taskCount: store.selection == .tasks ? store.openTaskCount(for: note) : nil
                            ) {
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

    private var emptyStateIcon: String {
        switch store.selection {
        case .tasks: return "checklist"
        case .trash: return "trash"
        default: return "tray"
        }
    }

    private var emptyStateTitle: String {
        switch store.selection {
        case .tasks: return "No open tasks"
        case .trash: return "Trash is empty"
        default: return "No notes"
        }
    }
}

private struct NoteRow: View {
    @EnvironmentObject var store: KansoStore
    let note: NoteDto
    let selected: Bool
    let taskCount: Int?
    let action: () -> Void

    var body: some View {
        VStack(spacing: 0) {
            Button(action: action) {
                VStack(alignment: .leading, spacing: 4) {
                    HStack(spacing: 6) {
                        if note.pinned {
                            Image(systemName: "pin.fill").font(.system(size: 9)).foregroundStyle(Theme.accent)
                        }
                        Circle()
                            .fill(statusColor(note.status))
                            .frame(width: 7, height: 7)
                        Text(note.title.isEmpty ? "Untitled" : note.title)
                            .font(.system(size: 13, weight: .semibold))
                            .foregroundStyle(Theme.textPrimary)
                            .lineLimit(1)
                        Spacer(minLength: 4)
                        if let taskCount, taskCount > 0 {
                            Label("\(taskCount)", systemImage: "checklist")
                                .font(.system(size: 10, weight: .medium))
                                .foregroundStyle(Theme.textMuted)
                                .labelStyle(.titleAndIcon)
                        }
                    }
                    Text(excerpt(for: note))
                        .font(.system(size: 11))
                        .foregroundStyle(selected ? Theme.textSecondary : Theme.textMuted)
                        .lineLimit(1)
                    HStack(spacing: 6) {
                        Text(shortDate(note.updatedAt))
                            .font(.system(size: 10, weight: .medium))
                            .foregroundStyle(Theme.textMuted)
                        if note.favorite {
                            Image(systemName: "star.fill")
                                .font(.system(size: 8))
                                .foregroundStyle(Theme.warning)
                        }
                    }
                }
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.horizontal, 14)
                .padding(.vertical, 10)
                .background(alignment: .leading) {
                    Rectangle()
                        .fill(selected ? Theme.accent : Color.clear)
                        .frame(width: 3)
                }
                .background(
                    RoundedRectangle(cornerRadius: 7)
                        .fill(selected ? Theme.accent.opacity(0.14) : Color.clear)
                )
                .overlay(
                    RoundedRectangle(cornerRadius: 7)
                        .stroke(selected ? Theme.accent.opacity(0.22) : Color.clear)
                )
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)
            .contextMenu {
                Menu {
                    ForEach(store.notebooks, id: \.id) { notebook in
                        Button(notebook.name) {
                            store.moveNote(note.id, toNotebook: notebook.id)
                        }
                        .disabled(notebook.id == note.notebookId)
                    }
                } label: {
                    Label("Move To", systemImage: "folder")
                }
                Button {
                    store.setPinned(note.id, pinned: !note.pinned)
                } label: {
                    Label(note.pinned ? "Unpin" : "Pin", systemImage: note.pinned ? "pin.slash" : "pin")
                }
                Button {
                    store.setFavorite(note.id, favorite: !note.favorite)
                } label: {
                    Label(note.favorite ? "Remove Favorite" : "Favorite", systemImage: note.favorite ? "star.slash" : "star")
                }
                Divider()
                Button(role: .destructive) {
                    store.deleteNote(note.id)
                } label: {
                    Label("Delete", systemImage: "trash")
                }
            }
            Divider().overlay(Theme.divider.opacity(selected ? 0 : 0.45)).padding(.leading, 14)
        }
        .padding(.horizontal, 6)
    }

    private func excerpt(for note: NoteDto) -> String {
        var lines = note.bodyMarkdown.components(separatedBy: .newlines)
        if let firstContentIndex = lines.firstIndex(where: { !$0.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty }),
           titleText(fromMarkdownLine: lines[firstContentIndex]) == note.title {
            lines.remove(at: firstContentIndex)
        }

        let flat = lines.joined(separator: " ")
            .trimmingCharacters(in: .whitespaces)
        return flat.isEmpty ? "No additional text" : flat
    }

    private func titleText(fromMarkdownLine line: String) -> String? {
        var text = line.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !text.isEmpty else { return nil }
        while text.hasPrefix(">") {
            text.removeFirst()
            text = text.trimmingCharacters(in: .whitespacesAndNewlines)
        }
        if let range = text.range(of: #"^#{1,6}\s+"#, options: .regularExpression) {
            text.removeSubrange(range)
        } else {
            return nil
        }
        if let range = text.range(of: #"\s+#+$"#, options: .regularExpression) {
            text.removeSubrange(range)
        }
        text = text.trimmingCharacters(in: .whitespacesAndNewlines)
        return text.isEmpty ? nil : text
    }

    private func statusColor(_ status: String) -> Color {
        switch status {
        case "completed": return Theme.success
        case "on_hold": return Theme.warning
        case "dropped": return Theme.textMuted
        default: return Theme.accent
        }
    }
}

// MARK: - Editor

private struct EditorView: View {
    @EnvironmentObject var store: KansoStore

    var body: some View {
        Group {
            if let note = store.note(store.selectedNoteId) {
                if store.isTrashNote(note.id) {
                    TrashNoteView(note: note).id(note.id)
                } else {
                    NoteEditor(note: note).id(note.id)
                }
            } else {
                EmptyEditorState()
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Theme.editor)
    }
}

private struct TrashNoteView: View {
    @EnvironmentObject var store: KansoStore
    let note: NoteDto
    @State private var confirmingPurge = false

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack(spacing: 12) {
                VStack(alignment: .leading, spacing: 5) {
                    Text(note.title.isEmpty ? "Untitled" : note.title)
                        .font(.system(size: 20, weight: .semibold))
                        .foregroundStyle(Theme.textPrimary)
                        .lineLimit(1)
                    Label("In Trash", systemImage: "trash")
                        .font(.system(size: 11))
                        .foregroundStyle(Theme.textMuted)
                }
                Spacer()
                Button {
                    store.restoreNote(note.id)
                } label: {
                    Label("Restore", systemImage: "arrow.uturn.backward")
                }
                .buttonStyle(.borderless)
                .foregroundStyle(Theme.textSecondary)
                Button(role: .destructive) {
                    confirmingPurge = true
                } label: {
                    Label("Delete Forever", systemImage: "trash.slash")
                }
                .buttonStyle(.borderless)
                .foregroundStyle(Theme.warning)
            }
            .padding(.horizontal, 26)
            .padding(.top, 22)
            .padding(.bottom, 14)

            Divider().overlay(Theme.divider)

            ScrollView {
                Text(note.bodyMarkdown.isEmpty ? "No additional text" : note.bodyMarkdown)
                    .font(.system(size: 15, design: .monospaced))
                    .foregroundStyle(Theme.textSecondary)
                    .frame(maxWidth: .infinity, alignment: .topLeading)
                    .padding(26)
                    .textSelection(.enabled)
            }
        }
        .alert("Delete forever?", isPresented: $confirmingPurge) {
            Button("Delete Forever", role: .destructive) {
                store.purgeNote(note.id)
            }
            Button("Cancel", role: .cancel) {}
        } message: {
            Text("This removes the note from Trash. Synced devices keep the deletion record so it will not come back.")
        }
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
    @State private var titleDraft: String = ""
    @State private var previewHTML: String = ""
    @State private var showingTagEditor = false
    @State private var newTagName = ""
    @State private var showingSketchSheet = false
    @State private var sketchStrokes: [InkStroke] = []
    @State private var editingSketch: SketchDto?
    @State private var showingAttachError = false
    @State private var showingHistorySheet = false
    @State private var showingShareSheet = false
    @State private var selectedInsightSection: NoteInsightSection?
    @AppStorage("KansoEditorSplitFraction") private var splitFraction = 0.5
    @State private var splitDragStartFraction: Double?

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack(spacing: 12) {
                TextField("Untitled", text: $titleDraft)
                    .textFieldStyle(.plain)
                    .font(.system(size: 20, weight: .semibold))
                    .foregroundStyle(Theme.textPrimary)
                    .onSubmit { commitTitle() }
                Spacer()
                EditorModeControl(selection: $store.editorMode)
                Button {
                    store.setPinned(note.id, pinned: !note.pinned)
                } label: {
                    Image(systemName: note.pinned ? "pin.fill" : "pin")
                        .font(.system(size: 13))
                        .foregroundStyle(note.pinned ? Theme.accent : Theme.textMuted)
                }
                .buttonStyle(.plain)
                .help(note.pinned ? "Unpin note" : "Pin note")
                Button {
                    store.setFavorite(note.id, favorite: !note.favorite)
                } label: {
                    Image(systemName: note.favorite ? "star.fill" : "star")
                        .font(.system(size: 13))
                        .foregroundStyle(note.favorite ? Theme.warning : Theme.textMuted)
                }
                .buttonStyle(.plain)
                .help(note.favorite ? "Remove favorite" : "Favorite note")
                Menu {
                    ForEach(store.notebooks, id: \.id) { notebook in
                        Button(notebook.name) {
                            store.moveNote(note.id, toNotebook: notebook.id)
                        }
                        .disabled(notebook.id == note.notebookId)
                    }
                } label: {
                    Image(systemName: "folder")
                        .font(.system(size: 13))
                        .foregroundStyle(Theme.textMuted)
                }
                .menuStyle(.borderlessButton)
                .fixedSize()
                .help("Move note")
                Button {
                    showingTagEditor = true
                } label: {
                    Image(systemName: "tag")
                        .font(.system(size: 13))
                        .foregroundStyle(Theme.textMuted)
                }
                .buttonStyle(.plain)
                .help("Tags")
                .popover(isPresented: $showingTagEditor) {
                    TagEditorPopover(note: note, newTagName: $newTagName)
                        .environmentObject(store)
                }
                Button {
                    showingHistorySheet = true
                } label: {
                    Image(systemName: "clock.arrow.circlepath")
                        .font(.system(size: 13))
                        .foregroundStyle(hasConflicts ? Theme.warning : Theme.textMuted)
                }
                .buttonStyle(.plain)
                .help("History and conflicts")
                Button {
                    showingShareSheet = true
                } label: {
                    Image(systemName: "person.2")
                        .font(.system(size: 13))
                        .foregroundStyle(Theme.textMuted)
                }
                .buttonStyle(.plain)
                .help("Share note")
                Button {
                    beginInsertingSketch()
                } label: {
                    Image(systemName: "scribble")
                        .font(.system(size: 14))
                        .foregroundStyle(Theme.textMuted)
                }
                .buttonStyle(.plain)
                .help("Insert sketch")
                Button {
                    attachFiles()
                } label: {
                    Image(systemName: "paperclip")
                        .font(.system(size: 13))
                        .foregroundStyle(Theme.textMuted)
                }
                .buttonStyle(.plain)
                .help("Attach file")
                Button { store.deleteNote(note.id) } label: {
                    Image(systemName: "trash").font(.system(size: 13)).foregroundStyle(Theme.textMuted)
                }
                .buttonStyle(.plain)
                .help("Delete note")
            }
            .padding(.horizontal, 26)
            .padding(.top, 22)
            .padding(.bottom, 10)

            NoteMetadataStrip(
                note: note,
                activeInsightSection: selectedInsightSection,
                onOpenInsight: toggleInsightSection,
                onOpenHistory: { showingHistorySheet = true }
            )
                .environmentObject(store)
                .padding(.horizontal, 26)
                .padding(.bottom, 10)

            Divider().overlay(Theme.divider)

            ZStack(alignment: .trailing) {
                editorBody
                    .frame(maxWidth: .infinity, maxHeight: .infinity)

                if selectedInsightSection != nil {
                    NoteInsightDrawer(
                        note: note,
                        draft: $draft,
                        selectedSection: $selectedInsightSection
                    )
                    .environmentObject(store)
                    .transition(.move(edge: .trailing).combined(with: .opacity))
                }
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
        .background(Theme.editor)
        .animation(.snappy(duration: 0.18), value: selectedInsightSection)
        .onAppear {
            draft = note.bodyMarkdown
            titleDraft = note.title
            refreshPreview()
        }
        .onChange(of: note.bodyMarkdown) { _, newBody in
            if draft != newBody {
                draft = newBody
            }
            refreshPreview()
        }
        .onChange(of: note.title) { _, newTitle in
            if titleDraft != newTitle {
                titleDraft = newTitle
            }
        }
        .onChange(of: store.pendingEditorAction) { _, action in
            handleEditorAction(action)
        }
        .onDisappear {
            commitTitle()
        }
        .sheet(isPresented: $showingSketchSheet, onDismiss: resetSketchSheet) {
            SketchCaptureSheet(
                heading: editingSketch == nil ? "Insert Sketch" : "Edit Sketch",
                helperText: editingSketch == nil
                    ? "Draw a block that stays referenced from Markdown."
                    : "Update the sketch stored in this Markdown block.",
                initialTitle: editingSketch?.title ?? "Sketch",
                showsTitleField: editingSketch == nil,
                strokes: $sketchStrokes
            ) { title in
                if let editingSketch {
                    if store.updateSketch(editingSketch.id, strokes: sketchStrokes, noteId: note.id) {
                        refreshPreview()
                    }
                } else {
                    if let nextBody = store.insertSketch(
                        noteId: note.id,
                        title: title,
                        strokes: sketchStrokes,
                        bodyMarkdown: draft
                    ) {
                        draft = nextBody
                        refreshPreview()
                    }
                }
                showingSketchSheet = false
            } onCancel: {
                showingSketchSheet = false
            }
        }
        .sheet(isPresented: $showingHistorySheet) {
            RevisionHistorySheet(note: note, draft: $draft)
                .environmentObject(store)
        }
        .sheet(isPresented: $showingShareSheet) {
            ShareMembersSheet(
                resourceType: "note",
                resourceId: note.id,
                title: note.title.isEmpty ? "Untitled" : note.title
            )
            .environmentObject(store)
        }
        .alert("Attachment failed", isPresented: $showingAttachError) {
            Button("OK", role: .cancel) {}
        } message: {
            Text(store.syncMessage)
        }
    }

    @ViewBuilder
    private var editorBody: some View {
        switch store.editorMode {
        case .edit:
            sourceEditor
        case .preview:
            markdownPreview
        case .split:
            GeometryReader { proxy in
                let handleWidth: CGFloat = 9
                let sourceWidth = splitSourceWidth(totalWidth: proxy.size.width, handleWidth: handleWidth)
                let previewWidth = max(proxy.size.width - sourceWidth - handleWidth, 0)
                HStack(spacing: 0) {
                    sourceEditor
                        .frame(width: sourceWidth)
                    EditorSplitResizeHandle()
                        .frame(width: handleWidth)
                        .gesture(
                            DragGesture(minimumDistance: 0)
                                .onChanged { value in
                                    let start = splitDragStartFraction ?? splitFraction
                                    splitDragStartFraction = start
                                    let delta = Double(value.translation.width / max(proxy.size.width, 1))
                                    splitFraction = clampedSplitFraction(start + delta)
                                }
                                .onEnded { _ in
                                    splitDragStartFraction = nil
                                }
                        )
                    markdownPreview
                        .frame(width: previewWidth)
                }
            }
        }
    }

    private var markdownPreview: some View {
        MarkdownPreviewView(
            html: previewHTML,
            markdown: draft,
            attachments: store.attachments(forNote: note.id),
            onOpenURL: handlePreviewURL,
            sketchPreview: { target in
                store.sketchPreviewData(matching: target, noteId: note.id)
            }
        )
    }

    private var sourceEditor: some View {
        TextEditor(text: $draft)
            .font(.system(size: 15, design: .monospaced))
            .lineSpacing(3)
            .foregroundStyle(Theme.textPrimary)
            .scrollContentBackground(.hidden)
            .background(Theme.editor)
            .padding(.horizontal, 22)
            .padding(.top, 10)
            .onChange(of: draft) { _, newValue in
                store.updateBody(note.id, newValue)
                refreshPreview()
            }
    }

    private func refreshPreview() {
        previewHTML = store.renderHTML(note.id)
    }

    private func handlePreviewURL(_ url: URL) {
        guard url.scheme?.localizedLowercase == "kanso" else {
            NSWorkspace.shared.open(url)
            return
        }

        guard let kind = url.host?.localizedLowercase,
              let target = previewTarget(from: url) else { return }

        switch kind {
        case "note", "embed":
            store.openOrCreateLinkedNote(named: target, preferredNotebookId: note.notebookId)
        case "sketch":
            beginEditingSketch(target: target)
        case "attachment":
            if let attachment = store.attachment(matching: target, noteId: note.id),
               let path = attachment.localPath {
                NSWorkspace.shared.open(URL(fileURLWithPath: path))
            }
        default:
            break
        }
    }

    private func previewTarget(from url: URL) -> String? {
        guard let host = url.host else { return nil }
        let prefix = "kanso://\(host)/"
        let raw: String
        if url.absoluteString.hasPrefix(prefix) {
            raw = String(url.absoluteString.dropFirst(prefix.count))
        } else {
            raw = String(url.path.dropFirst())
        }
        let decoded = (raw.removingPercentEncoding ?? raw)
            .trimmingCharacters(in: .whitespacesAndNewlines)
        return decoded.isEmpty ? nil : decoded
    }

    private var hasConflicts: Bool {
        !store.conflicts(forNote: note.id).isEmpty
    }

    private func attachFiles() {
        let panel = NSOpenPanel()
        panel.allowsMultipleSelection = true
        panel.canChooseFiles = true
        panel.canChooseDirectories = false
        panel.allowedContentTypes = [.item]

        guard panel.runModal() == .OK else { return }
        var nextBody = draft
        var attachedAny = false
        for url in panel.urls {
            if let updated = store.attachFile(url: url, noteId: note.id, bodyMarkdown: nextBody) {
                nextBody = updated
                attachedAny = true
            } else {
                showingAttachError = true
            }
        }
        if attachedAny {
            draft = nextBody
            refreshPreview()
        }
    }

    private func handleEditorAction(_ action: KansoEditorAction?) {
        guard let action else { return }
        defer { store.pendingEditorAction = nil }

        switch action {
        case .insertSketch:
            beginInsertingSketch()
        case .attachFile:
            attachFiles()
        case .showHistory:
            showingHistorySheet = true
        case .shareNote:
            showingShareSheet = true
        }
    }

    private func commitTitle() {
        guard titleDraft.trimmingCharacters(in: .whitespacesAndNewlines) != note.title else { return }
        store.updateTitle(note.id, titleDraft)
    }

    private func toggleInsightSection(_ section: NoteInsightSection) {
        selectedInsightSection = selectedInsightSection == section ? nil : section
    }

    private func beginInsertingSketch() {
        editingSketch = nil
        sketchStrokes = []
        showingSketchSheet = true
    }

    private func beginEditingSketch(target: String) {
        guard let (sketch, strokes) = store.sketchStrokes(matching: target, noteId: note.id) else {
            store.syncPhase = .error
            store.syncMessage = "Sketch is not available to edit"
            return
        }
        editingSketch = sketch
        sketchStrokes = strokes
        showingSketchSheet = true
    }

    private func resetSketchSheet() {
        sketchStrokes = []
        editingSketch = nil
    }

    private func splitSourceWidth(totalWidth: CGFloat, handleWidth: CGFloat) -> CGFloat {
        let availableWidth = max(totalWidth - handleWidth, 0)
        guard availableWidth > 0 else { return 0 }
        let minimumColumnWidth = min(260, availableWidth / 2)
        let proposedWidth = availableWidth * CGFloat(clampedSplitFraction(splitFraction))
        return min(max(proposedWidth, minimumColumnWidth), availableWidth - minimumColumnWidth)
    }

    private func clampedSplitFraction(_ value: Double) -> Double {
        min(max(value, 0.25), 0.75)
    }
}

private struct EditorSplitResizeHandle: View {
    var body: some View {
        ZStack {
            Rectangle().fill(Theme.divider)
            Capsule()
                .fill(Theme.textMuted.opacity(0.45))
                .frame(width: 2, height: 34)
        }
        .contentShape(Rectangle())
        .help("Drag to resize split preview")
    }
}

private struct NoteMetadataStrip: View {
    @EnvironmentObject var store: KansoStore
    let note: NoteDto
    let activeInsightSection: NoteInsightSection?
    let onOpenInsight: (NoteInsightSection) -> Void
    let onOpenHistory: () -> Void

    var body: some View {
        let noteTags = store.tags(forNote: note.id)
        let backlinkCount = store.backlinks(forNote: note.id).count
        let linkCount = store.outgoingLinks(forNote: note.id).count
        let taskCount = store.openTasks(for: note).count
        let attachmentCount = store.attachments(forNote: note.id).count
        let revisionCount = store.revisions(forNote: note.id).count
        let conflictCount = store.conflicts(forNote: note.id).count

        HStack(spacing: 8) {
            NoteStatusMenu(note: note)
                .environmentObject(store)
            if noteTags.isEmpty {
                Label("No tags", systemImage: "tag")
                    .font(.system(size: 11))
                    .foregroundStyle(Theme.textMuted)
            } else {
                ForEach(noteTags, id: \.id) { tag in
                    TagChip(tag: tag)
                }
            }
            Spacer(minLength: 12)
            InsightPill(
                icon: NoteInsightSection.backlinks.icon,
                text: "\(backlinkCount)",
                selected: activeInsightSection == .backlinks
            ) {
                onOpenInsight(.backlinks)
            }
            InsightPill(
                icon: NoteInsightSection.links.icon,
                text: "\(linkCount)",
                selected: activeInsightSection == .links
            ) {
                onOpenInsight(.links)
            }
            InsightPill(
                icon: NoteInsightSection.tasks.icon,
                text: "\(taskCount)",
                selected: activeInsightSection == .tasks
            ) {
                onOpenInsight(.tasks)
            }
            InsightPill(
                icon: NoteInsightSection.attachments.icon,
                text: "\(attachmentCount)",
                selected: activeInsightSection == .attachments
            ) {
                onOpenInsight(.attachments)
            }
            InsightPill(icon: conflictCount > 0 ? "exclamationmark.triangle" : "clock.arrow.circlepath",
                        text: conflictCount > 0 ? "\(conflictCount)" : "\(revisionCount)",
                        tone: conflictCount > 0 ? .warning : .muted) {
                onOpenHistory()
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }
}

private struct NoteStatusMenu: View {
    @EnvironmentObject var store: KansoStore
    let note: NoteDto

    var body: some View {
        Menu {
            statusButton("active", "Active", "circle")
            statusButton("on_hold", "On Hold", "pause.circle")
            statusButton("completed", "Completed", "checkmark.circle")
            statusButton("dropped", "Dropped", "xmark.circle")
        } label: {
            HStack(spacing: 5) {
                Circle().fill(color(for: note.status)).frame(width: 6, height: 6)
                Text(title(for: note.status))
                    .font(.system(size: 11, weight: .medium))
            }
            .foregroundStyle(Theme.textSecondary)
            .padding(.horizontal, 8)
            .padding(.vertical, 4)
            .background(RoundedRectangle(cornerRadius: 7).fill(Theme.elevated))
        }
        .menuStyle(.borderlessButton)
        .fixedSize()
    }

    private func statusButton(_ status: String, _ title: String, _ icon: String) -> some View {
        Button {
            store.setStatus(note.id, status: status)
        } label: {
            Label(title, systemImage: note.status == status ? "checkmark" : icon)
        }
    }

    private func title(for status: String) -> String {
        switch status {
        case "completed": return "Completed"
        case "on_hold": return "On Hold"
        case "dropped": return "Dropped"
        default: return "Active"
        }
    }

    private func color(for status: String) -> Color {
        switch status {
        case "completed": return Theme.success
        case "on_hold": return Theme.warning
        case "dropped": return Theme.textMuted
        default: return Theme.accent
        }
    }
}

private struct TagChip: View {
    let tag: TagDto

    var body: some View {
        HStack(spacing: 5) {
            Circle().fill(Theme.accent).frame(width: 6, height: 6)
            Text(tag.name)
                .font(.system(size: 11, weight: .medium))
                .foregroundStyle(Theme.textSecondary)
                .lineLimit(1)
        }
        .padding(.horizontal, 8)
        .padding(.vertical, 4)
        .background(RoundedRectangle(cornerRadius: 7).fill(Theme.elevated))
    }
}

private struct InsightPill: View {
    enum Tone {
        case muted
        case warning
    }

    let icon: String
    let text: String
    var tone: Tone = .muted
    var selected = false
    var action: (() -> Void)?

    var body: some View {
        if let action {
            Button(action: action) {
                label
            }
            .buttonStyle(.plain)
            .help(text)
        } else {
            label
        }
    }

    private var label: some View {
        HStack(spacing: 4) {
            Image(systemName: icon).font(.system(size: 10))
            Text(text).font(.system(size: 11, weight: .medium))
        }
        .foregroundStyle(foreground)
        .padding(.horizontal, 7)
        .padding(.vertical, 4)
        .background(RoundedRectangle(cornerRadius: 7).fill(background))
        .overlay(RoundedRectangle(cornerRadius: 7).stroke(border))
    }

    private var foreground: Color {
        if tone == .warning { return Theme.warning }
        return selected ? Theme.textPrimary : Theme.textMuted
    }

    private var background: Color {
        if tone == .warning { return Theme.warning.opacity(0.13) }
        return selected ? Theme.accent.opacity(0.18) : Theme.field
    }

    private var border: Color {
        selected ? Theme.accent.opacity(0.35) : Theme.divider.opacity(0.35)
    }
}

private struct TagEditorPopover: View {
    @EnvironmentObject var store: KansoStore
    let note: NoteDto
    @Binding var newTagName: String

    private var assignedTagIds: Set<String> {
        Set(store.tags(forNote: note.id).map(\.id))
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Tags")
                .font(.system(size: 15, weight: .semibold))
                .foregroundStyle(Theme.textPrimary)

            if store.tags.isEmpty {
                Text("Create your first tag for this note.")
                    .font(.system(size: 12))
                    .foregroundStyle(Theme.textMuted)
            } else {
                ScrollView {
                    VStack(spacing: 4) {
                        ForEach(store.tags, id: \.id) { tag in
                            let assigned = assignedTagIds.contains(tag.id)
                            Button {
                                store.setTag(tag, on: note.id, enabled: !assigned)
                            } label: {
                                HStack(spacing: 8) {
                                    Image(systemName: assigned ? "checkmark.circle.fill" : "circle")
                                        .font(.system(size: 12))
                                        .foregroundStyle(assigned ? Theme.accent : Theme.textMuted)
                                    Text(tag.name)
                                        .font(.system(size: 12))
                                        .foregroundStyle(Theme.textSecondary)
                                        .lineLimit(1)
                                    Spacer()
                                }
                                .padding(.horizontal, 8)
                                .padding(.vertical, 6)
                                .background(RoundedRectangle(cornerRadius: 7).fill(assigned ? Theme.accent.opacity(0.12) : Color.clear))
                            }
                            .buttonStyle(.plain)
                        }
                    }
                }
                .frame(maxHeight: 180)
            }

            HStack(spacing: 8) {
                TextField("New tag", text: $newTagName)
                    .textFieldStyle(.plain)
                    .font(.system(size: 12))
                    .foregroundStyle(Theme.textPrimary)
                    .padding(.horizontal, 9)
                    .padding(.vertical, 7)
                    .background(RoundedRectangle(cornerRadius: 7).fill(Theme.elevated))
                Button {
                    if let tag = store.createTag(named: newTagName) {
                        store.setTag(tag, on: note.id, enabled: true)
                        newTagName = ""
                    }
                } label: {
                    Image(systemName: "plus")
                        .font(.system(size: 12, weight: .semibold))
                        .frame(width: 28, height: 28)
                }
                .buttonStyle(.plain)
                .foregroundStyle(Theme.textPrimary)
                .background(RoundedRectangle(cornerRadius: 7).fill(Theme.accent))
            }
        }
        .padding(14)
        .frame(width: 280)
        .background(Theme.noteList)
    }
}

private struct RevisionHistorySheet: View {
    @EnvironmentObject var store: KansoStore
    @Environment(\.dismiss) private var dismiss
    let note: NoteDto
    @Binding var draft: String
    @State private var selectedRevisionId: String?

    private var revisions: [RevisionDto] {
        store.revisions(forNote: note.id)
    }

    private var conflicts: [RevisionDto] {
        store.conflicts(forNote: note.id)
    }

    private var selectedRevision: RevisionDto? {
        revisions.first { $0.id == selectedRevisionId } ?? revisions.first
    }

    var body: some View {
        HStack(spacing: 0) {
            VStack(alignment: .leading, spacing: 12) {
                VStack(alignment: .leading, spacing: 4) {
                    Text("History")
                        .font(.system(size: 22, weight: .semibold))
                        .foregroundStyle(Theme.textPrimary)
                    Text(note.title.isEmpty ? "Untitled" : note.title)
                        .font(.system(size: 12))
                        .foregroundStyle(Theme.textMuted)
                        .lineLimit(1)
                }

                if !conflicts.isEmpty {
                    HStack(spacing: 7) {
                        Image(systemName: "exclamationmark.triangle")
                        Text("\(conflicts.count) conflict\(conflicts.count == 1 ? "" : "s") preserved")
                    }
                    .font(.system(size: 11, weight: .medium))
                    .foregroundStyle(Theme.warning)
                    .padding(.horizontal, 9)
                    .padding(.vertical, 7)
                    .background(RoundedRectangle(cornerRadius: 7).fill(Theme.warning.opacity(0.12)))
                }

                if revisions.isEmpty {
                    EmptyInsightText("No revisions yet")
                    Spacer()
                } else {
                    ScrollView {
                        VStack(spacing: 6) {
                            ForEach(revisions, id: \.id) { revision in
                                RevisionRow(
                                    revision: revision,
                                    selected: revision.id == selectedRevision?.id
                                ) {
                                    selectedRevisionId = revision.id
                                }
                            }
                        }
                    }
                }
            }
            .padding(18)
            .frame(width: 280)
            .frame(maxHeight: .infinity, alignment: .topLeading)
            .background(Theme.sidebar)

            Divider().overlay(Theme.divider)

            VStack(alignment: .leading, spacing: 0) {
                HStack(spacing: 10) {
                    VStack(alignment: .leading, spacing: 3) {
                        Text(selectedRevision.map(title(for:)) ?? "No revision selected")
                            .font(.system(size: 17, weight: .semibold))
                            .foregroundStyle(Theme.textPrimary)
                        Text(selectedRevision.map(subtitle(for:)) ?? "Edit the note to create snapshots.")
                            .font(.system(size: 11))
                            .foregroundStyle(Theme.textMuted)
                    }
                    Spacer()
                    Button("Close") { dismiss() }
                        .buttonStyle(.borderless)
                        .foregroundStyle(Theme.textSecondary)
                    Button {
                        if let revision = selectedRevision,
                           let restored = store.restoreRevision(revision) {
                            draft = restored
                            dismiss()
                        }
                    } label: {
                        Label("Restore", systemImage: "arrow.uturn.backward")
                    }
                    .buttonStyle(.borderless)
                    .foregroundStyle(selectedRevision == nil ? Theme.textMuted : Theme.textPrimary)
                    .disabled(selectedRevision == nil)
                }
                .padding(.horizontal, 20)
                .padding(.top, 18)
                .padding(.bottom, 12)

                Divider().overlay(Theme.divider)

                ScrollView {
                    Text(selectedRevision?.bodyMarkdown ?? "")
                        .font(.system(size: 13, design: .monospaced))
                        .foregroundStyle(Theme.textSecondary)
                        .frame(maxWidth: .infinity, alignment: .topLeading)
                        .padding(20)
                        .textSelection(.enabled)
                }
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .background(Theme.editor)
        }
        .frame(width: 860, height: 580)
        .onAppear {
            selectedRevisionId = selectedRevision?.id
        }
    }

    private func title(for revision: RevisionDto) -> String {
        switch revision.source {
        case "conflict": return "Conflict copy"
        case "sync": return "Remote version"
        case "import": return "Imported version"
        case "agent": return "Agent edit"
        default: return "User edit"
        }
    }

    private func subtitle(for revision: RevisionDto) -> String {
        let reason = revision.reason?.trimmingCharacters(in: .whitespacesAndNewlines)
        if let reason, !reason.isEmpty {
            return "\(shortDate(revision.createdAt)) · \(reason)"
        }
        return "\(shortDate(revision.createdAt)) · \(revision.source)"
    }
}

private struct RevisionRow: View {
    let revision: RevisionDto
    let selected: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            VStack(alignment: .leading, spacing: 4) {
                HStack(spacing: 6) {
                    Image(systemName: revision.source == "conflict" ? "exclamationmark.triangle" : "clock.arrow.circlepath")
                        .font(.system(size: 10))
                        .foregroundStyle(revision.source == "conflict" ? Theme.warning : Theme.textMuted)
                    Text(title)
                        .font(.system(size: 12, weight: .semibold))
                        .foregroundStyle(Theme.textPrimary)
                        .lineLimit(1)
                    Spacer()
                    Text(shortDate(revision.createdAt))
                        .font(.system(size: 10))
                        .foregroundStyle(Theme.textMuted)
                }
                Text(excerpt)
                    .font(.system(size: 10))
                    .foregroundStyle(Theme.textMuted)
                    .lineLimit(2)
            }
            .padding(9)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(RoundedRectangle(cornerRadius: 7).fill(selected ? Theme.elevated : Color.clear))
        }
        .buttonStyle(.plain)
    }

    private var title: String {
        switch revision.source {
        case "conflict": return "Conflict"
        case "sync": return "Remote"
        case "import": return "Import"
        case "agent": return "Agent"
        default: return "Edit"
        }
    }

    private var excerpt: String {
        let flat = revision.bodyMarkdown
            .replacingOccurrences(of: "\n", with: " ")
            .trimmingCharacters(in: .whitespacesAndNewlines)
        return flat.isEmpty ? "Empty note body" : flat
    }
}

private struct ShareMembersSheet: View {
    @EnvironmentObject var store: KansoStore
    @Environment(\.dismiss) private var dismiss
    let resourceType: String
    let resourceId: String
    let title: String

    @State private var members: [ShareMemberDto] = []
    @State private var email = ""
    @State private var role = "viewer"

    private let roles = ["viewer", "editor", "owner"]

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Share")
                        .font(.system(size: 22, weight: .semibold))
                        .foregroundStyle(Theme.textPrimary)
                    Text(title)
                        .font(.system(size: 12))
                        .foregroundStyle(Theme.textMuted)
                        .lineLimit(1)
                }
                Spacer()
                Button("Done") { dismiss() }
                    .buttonStyle(.borderless)
                    .foregroundStyle(Theme.textSecondary)
            }

            HStack(spacing: 10) {
                TextField("person@example.com", text: $email)
                    .textFieldStyle(.plain)
                    .font(.system(size: 13))
                    .foregroundStyle(Theme.textPrimary)
                    .padding(.horizontal, 10)
                    .padding(.vertical, 8)
                    .background(RoundedRectangle(cornerRadius: 7).fill(Theme.elevated))
                Picker("Role", selection: $role) {
                    ForEach(roles, id: \.self) { role in
                        Text(roleTitle(role)).tag(role)
                    }
                }
                .labelsHidden()
                .frame(width: 120)
                Button {
                    addMember()
                } label: {
                    Label("Invite", systemImage: "person.badge.plus")
                }
                .buttonStyle(.borderless)
                .foregroundStyle(Theme.textPrimary)
            }

            Divider().overlay(Theme.divider)

            if members.isEmpty {
                VStack(spacing: 8) {
                    Image(systemName: "person.2")
                        .font(.system(size: 24))
                        .foregroundStyle(Theme.textMuted)
                    Text("No members yet")
                        .font(.system(size: 12, weight: .medium))
                        .foregroundStyle(Theme.textMuted)
                }
                .frame(maxWidth: .infinity, minHeight: 140)
            } else {
                ScrollView {
                    VStack(spacing: 8) {
                        ForEach(members, id: \.id) { member in
                            ShareMemberRow(member: member) {
                                store.removeShareMember(member.id)
                                reload()
                            }
                        }
                    }
                }
            }
        }
        .padding(22)
        .frame(width: 560, height: 460)
        .background(Theme.editor)
        .onAppear(perform: reload)
    }

    private func addMember() {
        if store.addShareMember(
            resourceType: resourceType,
            resourceId: resourceId,
            email: email,
            role: role
        ) != nil {
            email = ""
            reload()
        }
    }

    private func reload() {
        members = store.shareMembers(resourceType: resourceType, resourceId: resourceId)
    }

    private func roleTitle(_ role: String) -> String {
        switch role {
        case "owner": return "Owner"
        case "editor": return "Editor"
        default: return "Viewer"
        }
    }
}

private struct ShareMemberRow: View {
    let member: ShareMemberDto
    let onRemove: () -> Void

    var body: some View {
        HStack(spacing: 10) {
            Image(systemName: icon)
                .font(.system(size: 13))
                .foregroundStyle(tone)
                .frame(width: 18)
            VStack(alignment: .leading, spacing: 2) {
                Text(member.email)
                    .font(.system(size: 13, weight: .semibold))
                    .foregroundStyle(Theme.textPrimary)
                    .lineLimit(1)
                Text("\(roleTitle(member.role)) · \(member.status)")
                    .font(.system(size: 10))
                    .foregroundStyle(Theme.textMuted)
            }
            Spacer()
            Button(role: .destructive, action: onRemove) {
                Image(systemName: "xmark.circle")
                    .font(.system(size: 12))
            }
            .buttonStyle(.plain)
            .foregroundStyle(Theme.textMuted)
            .help("Remove member")
        }
        .padding(10)
        .background(RoundedRectangle(cornerRadius: 8).fill(Theme.noteList))
    }

    private var icon: String {
        switch member.role {
        case "owner": return "crown"
        case "editor": return "pencil.circle"
        default: return "eye"
        }
    }

    private var tone: Color {
        switch member.role {
        case "owner": return Theme.warning
        case "editor": return Theme.accent
        default: return Theme.textMuted
        }
    }

    private func roleTitle(_ role: String) -> String {
        switch role {
        case "owner": return "Owner"
        case "editor": return "Editor"
        default: return "Viewer"
        }
    }
}

private enum SketchTool {
    case pencil
    case eraser
}

private struct SketchCaptureSheet: View {
    @Environment(\.dismiss) private var dismiss
    let heading: String
    let helperText: String
    let initialTitle: String
    let showsTitleField: Bool
    @Binding var strokes: [InkStroke]
    let onSave: (String?) -> Void
    let onCancel: () -> Void
    @State private var title: String
    @State private var selectedTool: SketchTool = .pencil
    @State private var selectedColor = ColorRgba(r: 20, g: 20, b: 20, a: 255)
    @State private var selectedWidth = 2.5
    @State private var undoStack: [[InkStroke]] = []
    @State private var redoStack: [[InkStroke]] = []

    private let colorPalette = [
        ColorRgba(r: 20, g: 20, b: 20, a: 255),
        ColorRgba(r: 94, g: 120, b: 149, a: 255),
        ColorRgba(r: 123, g: 170, b: 120, a: 255),
        ColorRgba(r: 196, g: 154, b: 90, a: 255),
        ColorRgba(r: 201, g: 110, b: 99, a: 255)
    ]

    init(
        heading: String = "Insert Sketch",
        helperText: String = "Draw a block that stays referenced from Markdown.",
        initialTitle: String = "Sketch",
        showsTitleField: Bool = true,
        strokes: Binding<[InkStroke]>,
        onSave: @escaping (String?) -> Void,
        onCancel: @escaping () -> Void
    ) {
        self.heading = heading
        self.helperText = helperText
        self.initialTitle = initialTitle
        self.showsTitleField = showsTitleField
        self._strokes = strokes
        self.onSave = onSave
        self.onCancel = onCancel
        self._title = State(initialValue: initialTitle)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text(heading)
                        .font(.system(size: 20, weight: .semibold))
                        .foregroundStyle(Theme.textPrimary)
                    Text(helperText)
                        .font(.system(size: 12))
                        .foregroundStyle(Theme.textMuted)
                }
                Spacer()
                Text("\(strokes.count) stroke\(strokes.count == 1 ? "" : "s")")
                    .font(.system(size: 11, weight: .medium))
                    .foregroundStyle(Theme.textMuted)
            }

            if showsTitleField {
                TextField("Title", text: $title)
                    .textFieldStyle(.plain)
                    .font(.system(size: 13))
                    .foregroundStyle(Theme.textPrimary)
                    .padding(.horizontal, 10)
                    .padding(.vertical, 8)
                    .background(RoundedRectangle(cornerRadius: 7).fill(Theme.elevated))
            } else {
                readOnlySketchTitle
            }

            ZStack(alignment: .topLeading) {
                InkCanvas(
                    strokes: $strokes,
                    color: selectedColor,
                    width: Float(selectedWidth),
                    isErasing: selectedTool == .eraser
                ) { previous, _ in
                    undoStack.append(previous)
                    redoStack.removeAll()
                }
                    .frame(minWidth: 620, idealWidth: 700, minHeight: 360, idealHeight: 420)
                    .clipShape(RoundedRectangle(cornerRadius: 8))
                    .overlay(RoundedRectangle(cornerRadius: 8).stroke(Theme.divider))

                sketchToolStrip
                    .padding(10)
            }

            HStack {
                Button {
                    undoStack.append(strokes)
                    redoStack.removeAll()
                    strokes.removeAll()
                } label: {
                    Label("Clear", systemImage: "eraser")
                }
                .buttonStyle(.borderless)
                .foregroundStyle(Theme.textSecondary)
                .disabled(strokes.isEmpty)

                Button {
                    undo()
                } label: {
                    Image(systemName: "arrow.uturn.backward")
                }
                .buttonStyle(.borderless)
                .foregroundStyle(undoStack.isEmpty ? Theme.textMuted : Theme.textSecondary)
                .disabled(undoStack.isEmpty)
                .help("Undo")

                Button {
                    redo()
                } label: {
                    Image(systemName: "arrow.uturn.forward")
                }
                .buttonStyle(.borderless)
                .foregroundStyle(redoStack.isEmpty ? Theme.textMuted : Theme.textSecondary)
                .disabled(redoStack.isEmpty)
                .help("Redo")

                Spacer()

                Button("Cancel") {
                    onCancel()
                    dismiss()
                }
                .buttonStyle(.borderless)
                .foregroundStyle(Theme.textSecondary)

                Button {
                    let trimmed = title.trimmingCharacters(in: .whitespacesAndNewlines)
                    onSave(trimmed.isEmpty ? nil : trimmed)
                    dismiss()
                } label: {
                    Label(showsTitleField ? "Insert" : "Save", systemImage: showsTitleField ? "plus" : "checkmark")
                }
                .buttonStyle(.borderless)
                .foregroundStyle(strokes.isEmpty ? Theme.textMuted : Theme.textPrimary)
                .disabled(strokes.isEmpty)
            }
        }
        .padding(20)
        .frame(width: 760)
        .background(Theme.editor)
    }

    private var readOnlySketchTitle: some View {
        HStack(spacing: 8) {
            Image(systemName: "scribble")
                .foregroundStyle(Theme.accent)
            Text(initialTitle.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty ? "Sketch" : initialTitle)
                .font(.system(size: 13, weight: .medium))
                .foregroundStyle(Theme.textSecondary)
            Spacer()
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 8)
        .background(RoundedRectangle(cornerRadius: 7).fill(Theme.elevated))
    }

    private var sketchToolStrip: some View {
        HStack(spacing: 9) {
            toolButton(.pencil, systemImage: "pencil", help: "Pencil")
            toolButton(.eraser, systemImage: "eraser", help: "Eraser")

            Divider()
                .frame(height: 18)
                .overlay(Theme.divider)

            ForEach(colorPalette, id: \.self) { color in
                Button {
                    selectedColor = color
                    selectedTool = .pencil
                } label: {
                    Circle()
                        .fill(swiftUIColor(color))
                        .frame(width: 18, height: 18)
                        .overlay(
                            Circle().stroke(
                                selectedColor == color && selectedTool == .pencil
                                    ? Theme.textPrimary
                                    : Theme.divider,
                                lineWidth: selectedColor == color && selectedTool == .pencil ? 2 : 1
                            )
                        )
                }
                .buttonStyle(.plain)
                .help("Ink color")
            }

            Slider(value: $selectedWidth, in: 1.5...10, step: 0.5)
                .frame(width: 110)
                .help("Stroke width")
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 7)
        .background(RoundedRectangle(cornerRadius: 7).fill(Theme.editor.opacity(0.9)))
    }

    private func toolButton(_ tool: SketchTool, systemImage: String, help: String) -> some View {
        Button {
            selectedTool = tool
        } label: {
            Image(systemName: systemImage)
                .font(.system(size: 12, weight: .semibold))
                .foregroundStyle(selectedTool == tool ? Theme.textPrimary : Theme.textMuted)
                .frame(width: 24, height: 24)
                .background(
                    RoundedRectangle(cornerRadius: 6)
                        .fill(selectedTool == tool ? Theme.accent.opacity(0.24) : Color.clear)
                )
        }
        .buttonStyle(.plain)
        .help(help)
    }

    private func swiftUIColor(_ color: ColorRgba) -> Color {
        Color(
            red: Double(color.r) / 255,
            green: Double(color.g) / 255,
            blue: Double(color.b) / 255,
            opacity: Double(color.a) / 255
        )
    }

    private func undo() {
        guard let previous = undoStack.popLast() else { return }
        redoStack.append(strokes)
        strokes = previous
    }

    private func redo() {
        guard let next = redoStack.popLast() else { return }
        undoStack.append(strokes)
        strokes = next
    }
}

private enum NoteInsightSection: String, CaseIterable, Identifiable, Equatable {
    case backlinks
    case links
    case tasks
    case attachments

    var id: String { rawValue }

    var title: String {
        switch self {
        case .backlinks: return "Backlinks"
        case .links: return "Links"
        case .tasks: return "Open Tasks"
        case .attachments: return "Attachments"
        }
    }

    var icon: String {
        switch self {
        case .backlinks: return "arrowshape.turn.up.left"
        case .links: return "link"
        case .tasks: return "checklist"
        case .attachments: return "paperclip"
        }
    }

    var emptyText: String {
        switch self {
        case .backlinks: return "No backlinks yet"
        case .links: return "No outgoing links"
        case .tasks: return "No open tasks"
        case .attachments: return "No attachments"
        }
    }
}

private struct NoteInsightDrawer: View {
    @EnvironmentObject var store: KansoStore
    let note: NoteDto
    @Binding var draft: String
    @Binding var selectedSection: NoteInsightSection?

    var body: some View {
        VStack(spacing: 0) {
            header
            sectionPicker
                .padding(.horizontal, 14)
                .padding(.bottom, 12)
            Divider().overlay(Theme.divider.opacity(0.8))
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 8) {
                    currentContent
                }
                .padding(14)
            }
        }
        .frame(width: 368)
        .frame(maxHeight: .infinity)
        .background(Theme.panelElevated)
        .overlay(alignment: .leading) {
            Rectangle()
                .fill(Theme.divider)
                .frame(width: 1)
        }
        .shadow(color: .black.opacity(0.28), radius: 28, x: -12, y: 0)
    }

    private var currentSection: NoteInsightSection {
        selectedSection ?? .backlinks
    }

    private var header: some View {
        HStack(spacing: 10) {
            Image(systemName: currentSection.icon)
                .font(.system(size: 15, weight: .semibold))
                .foregroundStyle(Theme.accent)
                .frame(width: 30, height: 30)
                .background(RoundedRectangle(cornerRadius: 7).fill(Theme.accent.opacity(0.15)))
            VStack(alignment: .leading, spacing: 2) {
                Text(currentSection.title)
                    .font(.system(size: 16, weight: .semibold))
                    .foregroundStyle(Theme.textPrimary)
                Text(note.title.isEmpty ? "Untitled" : note.title)
                    .font(.system(size: 11))
                    .foregroundStyle(Theme.textMuted)
                    .lineLimit(1)
            }
            Spacer()
            Button {
                selectedSection = nil
            } label: {
                Image(systemName: "xmark")
                    .font(.system(size: 11, weight: .semibold))
                    .foregroundStyle(Theme.textMuted)
                    .frame(width: 24, height: 24)
                    .background(RoundedRectangle(cornerRadius: 6).fill(Theme.field))
            }
            .buttonStyle(.plain)
            .help("Close drawer")
        }
        .padding(.horizontal, 14)
        .padding(.top, 14)
        .padding(.bottom, 12)
    }

    private var sectionPicker: some View {
        HStack(spacing: 4) {
            ForEach(NoteInsightSection.allCases) { section in
                Button {
                    selectedSection = section
                } label: {
                    Image(systemName: section.icon)
                        .font(.system(size: 12, weight: .medium))
                        .foregroundStyle(currentSection == section ? Theme.textPrimary : Theme.textMuted)
                        .frame(maxWidth: .infinity)
                        .frame(height: 28)
                        .background(RoundedRectangle(cornerRadius: 6)
                            .fill(currentSection == section ? Theme.accent.opacity(0.2) : Color.clear))
                }
                .buttonStyle(.plain)
                .help(section.title)
            }
        }
        .padding(3)
        .background(RoundedRectangle(cornerRadius: 7).fill(Theme.field))
        .overlay(RoundedRectangle(cornerRadius: 7).stroke(Theme.divider.opacity(0.5)))
    }

    @ViewBuilder
    private var currentContent: some View {
        switch currentSection {
        case .backlinks:
            if backlinks.isEmpty {
                DrawerEmptyState(section: currentSection)
            } else {
                ForEach(backlinks, id: \.id) { linked in
                    Button {
                        store.selectedNoteId = linked.id
                        selectedSection = nil
                    } label: {
                        InsightRow(icon: "doc.text", text: linked.title)
                    }
                    .buttonStyle(.plain)
                }
            }
        case .links:
            if links.isEmpty {
                DrawerEmptyState(section: currentSection)
            } else {
                ForEach(Array(links.enumerated()), id: \.offset) { _, link in
                    if link.linkKind == "note" {
                        Button {
                            store.openOrCreateLinkedNote(
                                named: link.targetRef,
                                preferredNotebookId: note.notebookId
                            )
                            selectedSection = nil
                        } label: {
                            InsightRow(icon: icon(for: link.linkKind), text: link.targetRef)
                        }
                        .buttonStyle(.plain)
                        .help("Open or create linked note")
                    } else {
                        InsightRow(icon: icon(for: link.linkKind), text: link.targetRef)
                    }
                }
            }
        case .tasks:
            if tasks.isEmpty {
                DrawerEmptyState(section: currentSection)
            } else {
                ForEach(tasks, id: \.id) { task in
                    Button {
                        store.setTask(task, checked: true)
                    } label: {
                        InsightRow(icon: "circle", text: task.text)
                    }
                    .buttonStyle(.plain)
                    .help("Mark complete")
                }
            }
        case .attachments:
            if attachments.isEmpty {
                DrawerEmptyState(section: currentSection)
            } else {
                ForEach(attachments, id: \.id) { attachment in
                    AttachmentInsightRow(attachment: attachment) {
                        openAttachment(attachment)
                    } onDelete: {
                        draft = store.deleteAttachment(
                            attachment,
                            noteId: note.id,
                            bodyMarkdown: draft
                        )
                    }
                }
            }
        }
    }

    private var backlinks: [NoteDto] {
        store.backlinks(forNote: note.id)
    }

    private var links: [NoteLinkDto] {
        store.outgoingLinks(forNote: note.id)
    }

    private var tasks: [TaskItemDto] {
        store.openTasks(for: note)
    }

    private var attachments: [AttachmentDto] {
        store.attachments(forNote: note.id)
    }

    private func openAttachment(_ attachment: AttachmentDto) {
        guard let path = attachment.localPath else { return }
        NSWorkspace.shared.open(URL(fileURLWithPath: path))
    }

    private func icon(for kind: String) -> String {
        switch kind {
        case "sketch": return "scribble"
        case "attachment": return "paperclip"
        default: return "doc.text"
        }
    }
}

private struct DrawerEmptyState: View {
    let section: NoteInsightSection

    var body: some View {
        VStack(spacing: 9) {
            Image(systemName: section.icon)
                .font(.system(size: 18, weight: .semibold))
                .foregroundStyle(Theme.textMuted.opacity(0.8))
                .frame(width: 38, height: 38)
                .background(RoundedRectangle(cornerRadius: 8).fill(Theme.field))
            Text(section.emptyText)
                .font(.system(size: 12, weight: .medium))
                .foregroundStyle(Theme.textMuted)
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, 34)
    }
}

private struct InsightRow: View {
    let icon: String
    let text: String

    var body: some View {
        HStack(spacing: 10) {
            Image(systemName: icon)
                .font(.system(size: 11, weight: .semibold))
                .foregroundStyle(Theme.accent)
                .frame(width: 26, height: 26)
                .background(RoundedRectangle(cornerRadius: 6).fill(Theme.accent.opacity(0.12)))
            Text(text.isEmpty ? "Untitled" : text)
                .font(.system(size: 12, weight: .medium))
                .foregroundStyle(Theme.textSecondary)
                .lineLimit(1)
                .truncationMode(.tail)
            Spacer(minLength: 0)
        }
        .padding(.horizontal, 9)
        .padding(.vertical, 7)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(RoundedRectangle(cornerRadius: 7).fill(Theme.field.opacity(0.75)))
        .overlay(RoundedRectangle(cornerRadius: 7).stroke(Theme.divider.opacity(0.35)))
    }
}

private struct AttachmentInsightRow: View {
    let attachment: AttachmentDto
    let onOpen: () -> Void
    let onDelete: () -> Void

    var body: some View {
        HStack(spacing: 9) {
            Button(action: onOpen) {
                HStack(spacing: 10) {
                    Image(systemName: icon)
                        .font(.system(size: 11, weight: .semibold))
                        .foregroundStyle(Theme.accent)
                        .frame(width: 26, height: 26)
                        .background(RoundedRectangle(cornerRadius: 6).fill(Theme.accent.opacity(0.12)))
                    VStack(alignment: .leading, spacing: 2) {
                        Text(attachment.filename)
                            .font(.system(size: 12, weight: .medium))
                            .foregroundStyle(Theme.textSecondary)
                            .lineLimit(1)
                            .truncationMode(.middle)
                        Text(formatSize(attachment.sizeBytes))
                            .font(.system(size: 10))
                            .foregroundStyle(Theme.textMuted.opacity(0.75))
                            .lineLimit(1)
                    }
                }
            }
            .buttonStyle(.plain)
            .disabled(attachment.localPath == nil)
            .help(attachment.localPath == nil ? "File will download on sync" : "Open attachment")

            Spacer(minLength: 2)

            Button(action: onDelete) {
                Image(systemName: "xmark.circle")
                    .font(.system(size: 11))
                    .foregroundStyle(Theme.textMuted.opacity(0.8))
                    .frame(width: 22, height: 22)
            }
            .buttonStyle(.plain)
            .help("Remove attachment")
        }
        .padding(.horizontal, 9)
        .padding(.vertical, 7)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(RoundedRectangle(cornerRadius: 7).fill(Theme.field.opacity(0.75)))
        .overlay(RoundedRectangle(cornerRadius: 7).stroke(Theme.divider.opacity(0.35)))
    }

    private var icon: String {
        attachment.mimeType.hasPrefix("image/") ? "photo" : "doc"
    }

    private func formatSize(_ bytes: Int64) -> String {
        let units = ["B", "KB", "MB", "GB"]
        var value = Double(max(bytes, 0))
        var unit = units[0]
        for next in units.dropFirst() {
            if value < 1024 { break }
            value /= 1024
            unit = next
        }
        if unit == "B" {
            return "\(Int(value)) \(unit)"
        }
        return String(format: "%.1f %@", value, unit)
    }
}

private struct EmptyInsightText: View {
    let text: String

    init(_ text: String) {
        self.text = text
    }

    var body: some View {
        Text(text)
            .font(.system(size: 11))
            .foregroundStyle(Theme.textMuted.opacity(0.75))
            .lineLimit(1)
    }
}

private struct EditorModeControl: View {
    @Binding var selection: KansoEditorMode

    var body: some View {
        HStack(spacing: 2) {
            ForEach(KansoEditorMode.allCases) { mode in
                Button { selection = mode } label: {
                    Image(systemName: mode.symbol)
                        .font(.system(size: 12, weight: .medium))
                        .foregroundStyle(selection == mode ? Theme.textPrimary : Theme.textMuted)
                        .frame(width: 28, height: 24)
                        .background(RoundedRectangle(cornerRadius: 6)
                            .fill(selection == mode ? Theme.elevated : Color.clear))
                }
                .buttonStyle(.plain)
                .help(mode.title)
            }
        }
        .padding(2)
        .background(RoundedRectangle(cornerRadius: 7).fill(Theme.sidebar))
    }
}

// MARK: - Command palette

private struct CommandPaletteView: View {
    @EnvironmentObject var store: KansoStore
    @State private var query = ""

    private var trimmedQuery: String {
        query.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private var matchingNotes: [NoteDto] {
        let sorted = store.allNotes.sorted { $0.updatedAt > $1.updatedAt }
        guard !trimmedQuery.isEmpty else { return Array(sorted.prefix(8)) }
        let needle = trimmedQuery.localizedLowercase
        return Array(sorted.filter { note in
            note.title.localizedLowercase.contains(needle)
                || note.bodyMarkdown.localizedLowercase.contains(needle)
        }.prefix(8))
    }

    var body: some View {
        ZStack {
            Color.black.opacity(0.34)
                .ignoresSafeArea()
                .onTapGesture { dismiss() }

            VStack(alignment: .leading, spacing: 0) {
                HStack(spacing: 10) {
                    Image(systemName: "magnifyingglass")
                        .font(.system(size: 14))
                        .foregroundStyle(Theme.textMuted)
                    PaletteSearchField(text: $query, onSubmit: submit)
                        .frame(height: 24)
                    if !query.isEmpty {
                        Button {
                            query = ""
                        } label: {
                            Image(systemName: "xmark.circle.fill")
                                .font(.system(size: 13))
                                .foregroundStyle(Theme.textMuted)
                        }
                        .buttonStyle(.plain)
                    }
                }
                .padding(.horizontal, 16)
                .padding(.vertical, 13)

                Divider().overlay(Theme.divider)

                ScrollView {
                    VStack(spacing: 2) {
                        PaletteSectionLabel("Note actions")

                        PaletteRow(
                            icon: "square.and.pencil",
                            title: trimmedQuery.isEmpty ? "New note" : "New note: \(trimmedQuery)",
                            detail: "Create in current notebook"
                        ) {
                            store.createNote(title: trimmedQuery.isEmpty ? "Untitled" : trimmedQuery)
                            dismiss()
                        }

                        if !trimmedQuery.isEmpty {
                            PaletteRow(
                                icon: "magnifyingglass",
                                title: "Search all notes",
                                detail: "\(matchingNotes.count) matching note\(matchingNotes.count == 1 ? "" : "s")"
                            ) {
                                store.search = trimmedQuery
                                store.select(.all)
                                dismiss()
                            }
                        }

                        PaletteRow(
                            icon: "calendar.badge.plus",
                            title: "Daily note",
                            detail: "Open today's note"
                        ) {
                            store.createDailyNote()
                            dismiss()
                        }

                        PaletteRow(
                            icon: "checklist",
                            title: "Show open tasks",
                            detail: "\(store.openTaskItems.count) open task\(store.openTaskItems.count == 1 ? "" : "s")"
                        ) {
                            store.select(.tasks)
                            dismiss()
                        }

                        if store.note(store.selectedNoteId) != nil,
                           !(store.selectedNoteId.map(store.isTrashNote) ?? false) {
                            PaletteSectionLabel("Current note")

                            PaletteRow(
                                icon: "scribble",
                                title: "Insert sketch block",
                                detail: "Capture a first-party sketch and insert a Markdown reference"
                            ) {
                                store.requestEditorAction(.insertSketch)
                                dismiss()
                            }

                            PaletteRow(
                                icon: "paperclip",
                                title: "Attach file",
                                detail: "Copy a file into Kanso storage and insert an attachment block"
                            ) {
                                store.requestEditorAction(.attachFile)
                                dismiss()
                            }

                            PaletteRow(
                                icon: "clock.arrow.circlepath",
                                title: "History and conflicts",
                                detail: "Review revisions and restore a previous version"
                            ) {
                                store.requestEditorAction(.showHistory)
                                dismiss()
                            }

                            PaletteRow(
                                icon: "person.2",
                                title: "Share note",
                                detail: "Invite members as owner, editor, or viewer"
                            ) {
                                store.requestEditorAction(.shareNote)
                                dismiss()
                            }
                        }

                        PaletteSectionLabel("Editor")

                        PaletteRow(
                            icon: "pencil",
                            title: "Edit mode",
                            detail: "Show Markdown source"
                        ) {
                            store.editorMode = .edit
                            dismiss()
                        }

                        PaletteRow(
                            icon: "doc.richtext",
                            title: "Preview mode",
                            detail: "Show rendered Markdown"
                        ) {
                            store.editorMode = .preview
                            dismiss()
                        }

                        PaletteRow(
                            icon: "rectangle.split.2x1",
                            title: "Split preview",
                            detail: "Show source and rendered preview side by side"
                        ) {
                            store.editorMode = .split
                            dismiss()
                        }

                        PaletteSectionLabel("Agent and sync")

                        PaletteRow(
                            icon: "wand.and.stars",
                            title: "Run skill",
                            detail: store.skills.first(where: { $0.enabled })?.title ?? "Create an enabled skill first"
                        ) {
                            store.runFirstEnabledSkillOnCurrentNote()
                            dismiss()
                        }

                        PaletteRow(
                            icon: "point.3.connected.trianglepath.dotted",
                            title: "Configure MCP access",
                            detail: "Approve clients and grant scoped capabilities"
                        ) {
                            store.requestSettings("mcp")
                            dismiss()
                        }

                        PaletteRow(
                            icon: "books.vertical",
                            title: "Skills library",
                            detail: "Create and edit Markdown-defined skills"
                        ) {
                            store.requestSettings("skills")
                            dismiss()
                        }

                        PaletteRow(
                            icon: "arrow.triangle.2.circlepath",
                            title: "Sync settings",
                            detail: store.isSyncConfigured
                                ? store.syncMessage
                                : "Sign in and enable backup encryption"
                        ) {
                            store.requestSettings("sync")
                            dismiss()
                        }

                        if store.isSyncConfigured {
                            PaletteRow(
                                icon: "arrow.triangle.2.circlepath",
                                title: "Sync now",
                                detail: store.syncMessage
                            ) {
                                store.syncNow()
                                dismiss()
                            }
                        }

                        PaletteSectionLabel("Matching notes")

                        ForEach(matchingNotes, id: \.id) { note in
                            PaletteRow(
                                icon: note.pinned ? "pin.fill" : "doc.text",
                                title: note.title.isEmpty ? "Untitled" : note.title,
                                detail: paletteDetail(for: note)
                            ) {
                                store.openNote(note.id)
                                dismiss()
                            }
                        }

                        if matchingNotes.isEmpty, trimmedQuery.isEmpty {
                            PaletteEmptyRow()
                        }
                    }
                    .padding(8)
                }
                .frame(maxHeight: 420)
            }
            .frame(width: 560)
            .background(RoundedRectangle(cornerRadius: 8).fill(Theme.noteList))
            .overlay(RoundedRectangle(cornerRadius: 8).stroke(Theme.divider))
            .shadow(color: .black.opacity(0.32), radius: 26, y: 18)
        }
        .onExitCommand { dismiss() }
    }

    private func submit() {
        if let first = matchingNotes.first {
            store.openNote(first.id)
        } else if !trimmedQuery.isEmpty {
            store.createNote(title: trimmedQuery)
        } else {
            store.createNote()
        }
        dismiss()
    }

    private func dismiss() {
        store.isCommandPalettePresented = false
    }

    private func paletteDetail(for note: NoteDto) -> String {
        let notebook = store.notebooks.first { $0.id == note.notebookId }?.name ?? "Notebook"
        let body = note.bodyMarkdown
            .replacingOccurrences(of: "\n", with: " ")
            .trimmingCharacters(in: .whitespacesAndNewlines)
        return body.isEmpty ? notebook : "\(notebook) - \(body)"
    }
}

private struct PaletteSearchField: NSViewRepresentable {
    @Binding var text: String
    let onSubmit: () -> Void

    func makeNSView(context: Context) -> NSTextField {
        let field = NSTextField()
        field.placeholderString = "Search or create"
        field.isBordered = false
        field.drawsBackground = false
        field.focusRingType = .none
        field.font = .systemFont(ofSize: 16)
        field.textColor = NSColor(calibratedRed: 0.945, green: 0.937, blue: 0.906, alpha: 1)
        field.delegate = context.coordinator

        DispatchQueue.main.async {
            field.window?.makeFirstResponder(field)
        }
        return field
    }

    func updateNSView(_ field: NSTextField, context: Context) {
        context.coordinator.parent = self
        if field.stringValue != text {
            field.stringValue = text
        }
    }

    func makeCoordinator() -> Coordinator {
        Coordinator(parent: self)
    }

    final class Coordinator: NSObject, NSTextFieldDelegate {
        var parent: PaletteSearchField

        init(parent: PaletteSearchField) {
            self.parent = parent
        }

        func controlTextDidChange(_ notification: Notification) {
            guard let field = notification.object as? NSTextField else { return }
            parent.text = field.stringValue
        }

        func control(
            _ control: NSControl,
            textView: NSTextView,
            doCommandBy commandSelector: Selector
        ) -> Bool {
            if commandSelector == #selector(NSResponder.insertNewline(_:)) {
                parent.text = textView.string
                parent.onSubmit()
                return true
            }
            return false
        }
    }
}

private struct PaletteSectionLabel: View {
    let title: String

    init(_ title: String) {
        self.title = title
    }

    var body: some View {
        Text(title)
            .font(.system(size: 10, weight: .semibold))
            .foregroundStyle(Theme.textMuted)
            .textCase(.uppercase)
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.horizontal, 10)
            .padding(.top, 9)
            .padding(.bottom, 3)
    }
}

private struct PaletteRow: View {
    let icon: String
    let title: String
    let detail: String
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 11) {
                Image(systemName: icon)
                    .font(.system(size: 13))
                    .foregroundStyle(Theme.accent)
                    .frame(width: 18)
                VStack(alignment: .leading, spacing: 2) {
                    Text(title)
                        .font(.system(size: 13, weight: .medium))
                        .foregroundStyle(Theme.textPrimary)
                        .lineLimit(1)
                    Text(detail)
                        .font(.system(size: 11))
                        .foregroundStyle(Theme.textMuted)
                        .lineLimit(1)
                }
                Spacer()
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 8)
            .background(RoundedRectangle(cornerRadius: 7).fill(Color.clear))
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
    }
}

private struct PaletteEmptyRow: View {
    var body: some View {
        HStack(spacing: 10) {
            Image(systemName: "tray")
                .font(.system(size: 13))
                .foregroundStyle(Theme.textMuted)
            Text("No notes yet")
                .font(.system(size: 12))
                .foregroundStyle(Theme.textMuted)
            Spacer()
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 12)
    }
}

// MARK: - Helpers

private func shortDate(_ millis: Int64) -> String {
    let date = Date(timeIntervalSince1970: Double(millis) / 1000)
    let formatter = DateFormatter()
    formatter.dateFormat = "MMM d"
    return formatter.string(from: date)
}
