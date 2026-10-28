// Kanso — CaptureIntents.swift
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import AppIntents
import UIKit

/// Opens Kanso on a new note, from Shortcuts, Siri, Spotlight or the Action
/// button.
struct NewNoteIntent: AppIntent {
  static let title: LocalizedStringResource = "New Note"
  static let description = IntentDescription("Starts a new note in Kanso.")
  static let openAppWhenRun = true

  @MainActor
  func perform() async throws -> some IntentResult {
    if let url = URL(string: "kanso://open/new") {
      await UIApplication.shared.open(url)
    }
    return .result()
  }
}

/// Opens Kanso on today's daily note.
struct OpenTodayIntent: AppIntent {
  static let title: LocalizedStringResource = "Open Today's Note"
  static let description = IntentDescription("Opens today's daily note in Kanso.")
  static let openAppWhenRun = true

  @MainActor
  func perform() async throws -> some IntentResult {
    if let url = URL(string: "kanso://open/daily") {
      await UIApplication.shared.open(url)
    }
    return .result()
  }
}

/// Adds text to the Inbox without opening Kanso.
struct AddToKansoIntent: AppIntent {
  static let title: LocalizedStringResource = "Add to Kanso"
  static let description = IntentDescription("Saves text as a note in your Kanso Inbox.")
  static let openAppWhenRun = false

  @Parameter(title: "Text")
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

  /// Writes the text as a drop (a link on its own goes in the drop's `url`)
  /// and asks the library to save it. Returns a sentence about what happened.
  private static func save(_ text: String) -> String {
    guard let location = KansoLibraryLocation.shared() else {
      return "Kanso couldn't open its library."
    }
    let link = URL(string: text).flatMap { url in
      ["http", "https"].contains(url.scheme?.lowercased() ?? "") ? url.absoluteString : nil
    }
    do {
      try DropWriter.write(
        in: location.inbox, text: link == nil ? text : nil, url: link)
    } catch {
      return "Kanso couldn't save this."
    }
    return KansoCore.summary(of: KansoCore.captureInbox(in: location))
  }
}

/// The shortcuts Kanso offers to Siri, Spotlight and the Shortcuts app.
struct KansoShortcuts: AppShortcutsProvider {
  static var appShortcuts: [AppShortcut] {
    AppShortcut(
      intent: NewNoteIntent(),
      phrases: [
        "New note in \(.applicationName)",
        "Start a note in \(.applicationName)",
      ],
      shortTitle: "New Note",
      systemImageName: "square.and.pencil")
    AppShortcut(
      intent: OpenTodayIntent(),
      phrases: [
        "Open today's note in \(.applicationName)",
        "Open \(.applicationName) today",
      ],
      shortTitle: "Open Today's Note",
      systemImageName: "calendar")
    AppShortcut(
      intent: AddToKansoIntent(),
      phrases: [
        "Add to \(.applicationName)",
        "Save a note in \(.applicationName)",
      ],
      shortTitle: "Add to Kanso",
      systemImageName: "tray.and.arrow.down")
  }
}
