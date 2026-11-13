// Kanso — KansoIntents.swift
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import AppIntents
import Foundation
import UIKit

// The phone's four system actions (#390), the same as Raycast's commands on
// the desktop: "Capture in Kanso", "Add Kanso task", "Search Kanso" and
// "Open Kanso note". They show up in Spotlight, Shortcuts, Siri and the
// Action button through `KansoShortcuts` (CaptureIntents.swift).
//
// Each runs only on an unlocked phone (`.requiresAuthentication`). Search
// and Open read nothing a lock hides: Search only opens the app's search,
// and Open is offered no locked note, nor any note in a locked notebook
// (`kanso_intent_notes`); the app opens what they ask for behind its own
// locks.
//
// Capture and Add task never open the app. They write a drop
// (`mobile/docs/capture.md`) and save it at once with the Rust library,
// which reuses the app's open library when the app is running; when the
// library can't be opened here, the drop waits and the app saves it on its
// next launch, as it does shared items.

/// "Capture in Kanso": text saved as a note in the Inbox; its first line is
/// the title.
struct CaptureInKansoIntent: AppIntent {
  static let title: LocalizedStringResource = "Capture in Kanso"
  static let description = IntentDescription(
    "Saves text as a note in your Kanso Inbox. The first line becomes its title.")
  static let openAppWhenRun = false
  static let authenticationPolicy: IntentAuthenticationPolicy = .requiresAuthentication

  @Parameter(title: "Text", inputOptions: String.IntentInputOptions(multiline: true))
  var text: String

  func perform() async throws -> some IntentResult & ProvidesDialog {
    let trimmed = text.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmed.isEmpty else {
      return .result(dialog: "There was nothing to save.")
    }
    let message = await Task.detached(priority: .userInitiated) {
      AddToKansoIntent.save(trimmed)
    }.value
    return .result(dialog: IntentDialog(stringLiteral: message))
  }
}

/// "Add Kanso task": a task read by the desktop's parser (`film Pixel
/// review fri 3pm every week`), added to today's daily note or the Inbox's
/// task note, as the setting "Captured tasks go to" says.
struct AddKansoTaskIntent: AppIntent {
  static let title: LocalizedStringResource = "Add Kanso task"
  static let description = IntentDescription(
    "Adds a task to today's daily note or your Inbox. A date and repeat at the end are read, as in “film Pixel review fri 3pm every week”."
  )
  static let openAppWhenRun = false
  static let authenticationPolicy: IntentAuthenticationPolicy = .requiresAuthentication

  @Parameter(title: "Task")
  var text: String

  func perform() async throws -> some IntentResult & ProvidesDialog {
    let typed = text
    let message = await Task.detached(priority: .userInitiated) {
      AddKansoTaskIntent.add(typed, at: Date())
    }.value
    return .result(dialog: IntentDialog(stringLiteral: message))
  }

  /// Reads `text` at `now`, writes the task drop and saves it. Returns the
  /// sentence to say.
  static func add(_ text: String, at now: Date) -> String {
    guard let location = KansoLibraryLocation.shared() else {
      return "Kanso couldn't open its library."
    }
    let preview: TaskPreview
    switch KansoIntentCore.taskPreview(text, at: now) {
    case .unavailable:
      return "Kanso couldn't read that task. Open Kanso and add it there."
    case .noTask:
      return "There was no task to add."
    case .task(let read):
      preview = read
    }
    let (day, time) = KansoIntentCore.localDayAndTime(now)
    do {
      try DropWriter.writeTask(
        in: location.inbox, line: preview.line, day: day, time: time,
        target: KansoIntentCore.taskTarget(in: location))
    } catch {
      return "Kanso couldn't save this task."
    }
    // Saved now when the library opens here; otherwise the app adds it on
    // its next launch, and the confirmation still holds.
    guard let result = KansoCore.captureInbox(in: location) else {
      return preview.confirmation
    }
    if result.tasks?.contains(preview.line) == true || result.kept > 0 {
      return preview.confirmation
    }
    let problems = result.problems.joined(separator: " ")
    return problems.isEmpty ? preview.confirmation : problems
  }
}

/// "Search Kanso": opens the app's search on the words given.
struct SearchKansoIntent: AppIntent {
  static let title: LocalizedStringResource = "Search Kanso"
  static let description = IntentDescription("Searches your notes in Kanso.")
  static let openAppWhenRun = true
  static let authenticationPolicy: IntentAuthenticationPolicy = .requiresAuthentication

  @Parameter(title: "Search for")
  var query: String

  @MainActor
  func perform() async throws -> some IntentResult {
    if let url = KansoIntentCore.searchLink(query) {
      await UIApplication.shared.open(url)
    }
    return .result()
  }
}

/// "Open Kanso note": opens a note chosen from the recently edited ones.
struct OpenKansoNoteIntent: AppIntent {
  static let title: LocalizedStringResource = "Open Kanso note"
  static let description = IntentDescription("Opens a note in Kanso.")
  static let openAppWhenRun = true
  static let authenticationPolicy: IntentAuthenticationPolicy = .requiresAuthentication

  @Parameter(title: "Note")
  var note: KansoNoteEntity

  @MainActor
  func perform() async throws -> some IntentResult {
    if let url = KansoIntentCore.noteLink(note.id) {
      await UIApplication.shared.open(url)
    }
    return .result()
  }
}

/// A note as Shortcuts, Siri and Spotlight show it: its title and notebook.
/// Only notes no lock hides are ever made into one.
struct KansoNoteEntity: AppEntity, Decodable {
  static let typeDisplayRepresentation: TypeDisplayRepresentation = "Kanso Note"
  static let defaultQuery = KansoNoteQuery()

  let id: String
  let title: String
  let notebook: String

  var displayRepresentation: DisplayRepresentation {
    DisplayRepresentation(
      title: "\(title.isEmpty ? "Untitled" : title)",
      subtitle: notebook.isEmpty ? nil : "\(notebook)")
  }
}

/// The notes "Open Kanso note" offers, newest edit first.
struct KansoNoteQuery: EntityStringQuery {
  func entities(for identifiers: [KansoNoteEntity.ID]) async throws -> [KansoNoteEntity] {
    let wanted = Set(identifiers)
    return await KansoIntentCore.notes(matching: "", limit: 50).filter { wanted.contains($0.id) }
  }

  func entities(matching string: String) async throws -> [KansoNoteEntity] {
    await KansoIntentCore.notes(matching: string, limit: 30)
  }

  func suggestedEntities() async throws -> [KansoNoteEntity] {
    await KansoIntentCore.notes(matching: "", limit: 20)
  }
}

/// A task as the Rust parser read it (`kanso_task_preview`).
struct TaskPreview: Decodable, Equatable {
  /// `- [ ] film Pixel review @2026-11-14T15:00 every week`.
  var line: String
  var words: String
  var when: String?
  /// `Added: film Pixel review · Fri 14 Nov 15:00 · every week`.
  var confirmation: String
}

/// What reading a typed task gave.
enum TaskReading: Equatable {
  case task(TaskPreview)
  /// There are no words in the text.
  case noTask
  /// The library couldn't be loaded.
  case unavailable
}

/// The intents' calls into the Rust library and the small facts they need.
enum KansoIntentCore {
  private typealias PreviewFunction = @convention(c) (
    UnsafePointer<CChar>?, Int32, UInt32, UInt32, UInt32, UInt32, Bool
  ) -> UnsafeMutablePointer<CChar>?
  private typealias NotesFunction = @convention(c) (
    UnsafePointer<CChar>?, UnsafePointer<CChar>?, UInt32
  ) -> UnsafeMutablePointer<CChar>?

  /// Reads the task in `text` at the phone's local time `now`.
  static func taskPreview(_ text: String, at now: Date) -> TaskReading {
    guard let found = KansoCore.symbol("kanso_task_preview") else { return .unavailable }
    let preview = unsafeBitCast(found, to: PreviewFunction.self)
    let parts = Calendar.current.dateComponents([.year, .month, .day, .hour, .minute], from: now)
    let pointer = text.withCString { text in
      preview(
        text, Int32(parts.year ?? 1970), UInt32(parts.month ?? 1), UInt32(parts.day ?? 1),
        UInt32(parts.hour ?? 0), UInt32(parts.minute ?? 0), monthFirst())
    }
    return reading(of: KansoCore.take(pointer))
  }

  /// What `kanso_task_preview`'s answer says: no answer is an unavailable
  /// library, `null` is no task.
  static func reading(of answer: Data?) -> TaskReading {
    guard let answer else { return .unavailable }
    let preview: TaskPreview?
    do {
      preview = try JSONDecoder().decode(TaskPreview?.self, from: answer)
    } catch {
      return .unavailable
    }
    guard let preview else { return .noTask }
    return .task(preview)
  }

  /// The notes "Open Kanso note" may offer for `query`. None when the
  /// library isn't here yet or can't be read. Blocks a background thread.
  static func notes(matching query: String, limit: UInt32) async -> [KansoNoteEntity] {
    await Task.detached(priority: .userInitiated) {
      guard
        let location = KansoLibraryLocation.shared(), location.databaseExists,
        let found = KansoCore.symbol("kanso_intent_notes")
      else { return [] }
      let notes = unsafeBitCast(found, to: NotesFunction.self)
      let pointer = location.database.path.withCString { path in
        query.withCString { query in notes(path, query, limit) }
      }
      return decodeNotes(KansoCore.take(pointer))
    }.value
  }

  /// The notes in `kanso_intent_notes`'s answer.
  static func decodeNotes(_ answer: Data?) -> [KansoNoteEntity] {
    struct Answer: Decodable { var notes: [KansoNoteEntity] }
    guard let answer, let decoded = try? JSONDecoder().decode(Answer.self, from: answer) else {
      return []
    }
    return decoded.notes
  }

  /// `kanso://open/search?q=…`, or `nil` for no words.
  static func searchLink(_ query: String) -> URL? {
    let words = query.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !words.isEmpty else { return URL(string: "kanso://open/search") }
    var link = URLComponents()
    link.scheme = "kanso"
    link.host = "open"
    link.path = "/search"
    link.queryItems = [URLQueryItem(name: "q", value: words)]
    return link.url
  }

  /// `kanso://open/note/<id>`, the link `deep_links.dart` reads.
  static func noteLink(_ id: String) -> URL? {
    guard !id.isEmpty, !id.contains("/"),
      let segment = id.addingPercentEncoding(withAllowedCharacters: .urlPathAllowed)
    else { return nil }
    return URL(string: "kanso://open/note/\(segment)")
  }

  /// The phone's local date (`YYYY-MM-DD`) and time (`HH:MM`) at `now`.
  static func localDayAndTime(_ now: Date, calendar: Calendar = .current) -> (String, String) {
    let parts = calendar.dateComponents([.year, .month, .day, .hour, .minute], from: now)
    let day = String(
      format: "%04d-%02d-%02d", parts.year ?? 1970, parts.month ?? 1, parts.day ?? 1)
    let time = String(format: "%02d:%02d", parts.hour ?? 0, parts.minute ?? 0)
    return (day, time)
  }

  /// Whether `11/14` reads as November 14th: the regions the app's
  /// `monthFirstFor` names.
  static func monthFirst(region: String? = Locale.current.region?.identifier) -> Bool {
    ["US", "PH", "FM", "GU", "MH", "PW"].contains(region?.uppercased() ?? "")
  }

  /// Where captured tasks go: the app's setting `captureTaskTarget` in
  /// `preferences.json`, `dailyNote` when it isn't set.
  static func taskTarget(in location: KansoLibraryLocation) -> String {
    let file = location.folder.appendingPathComponent("preferences.json")
    return taskTarget(from: try? Data(contentsOf: file))
  }

  static func taskTarget(from preferences: Data?) -> String {
    guard
      let preferences,
      let json = try? JSONSerialization.jsonObject(with: preferences) as? [String: Any],
      let target = json["captureTaskTarget"] as? String, target == "inbox"
    else { return "dailyNote" }
    return target
  }
}
