// Kanso — KansoLibraryLocation.swift
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import Foundation

/// Where the library lives on this phone, for everything that opens it: the
/// app, the share extension, the widgets and App Intents.
///
/// The folder is in the App Group's container so all of them see the same
/// files (see `mobile/docs/capture.md`).
struct KansoLibraryLocation {
  /// The App Group every Kanso target on this phone belongs to.
  static let appGroup = "group.app.kanso.Kanso"

  /// The library's folder: `Library/Application Support/kanso/`.
  let folder: URL

  /// The library's database. It is missing until the app has run once with
  /// the build that keeps the library in the App Group.
  var database: URL { folder.appendingPathComponent("library.db") }

  /// Where finished and unfinished drops wait for the library.
  var inbox: URL { folder.appendingPathComponent("inbox", isDirectory: true) }

  /// The widgets' colors, written by the app.
  var widgetColors: URL { folder.appendingPathComponent("widget.json") }

  /// Whether the app has made the library here yet.
  var databaseExists: Bool {
    FileManager.default.fileExists(atPath: database.path)
  }

  /// The location inside the App Group's container, or `nil` when this
  /// process isn't part of the group (an unsigned build, for one).
  static func shared() -> KansoLibraryLocation? {
    guard
      let container = FileManager.default.containerURL(
        forSecurityApplicationGroupIdentifier: appGroup)
    else { return nil }
    return KansoLibraryLocation(folder: location(in: container))
  }

  /// The library's folder inside a container.
  static func location(in container: URL) -> URL {
    container
      .appendingPathComponent("Library", isDirectory: true)
      .appendingPathComponent("Application Support", isDirectory: true)
      .appendingPathComponent("kanso", isDirectory: true)
  }
}
