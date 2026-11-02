// Kanso — VoiceDropWriter.swift
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import Foundation

/// Writes one *voice drop*: `inbox/<id>.partial/` holds `rec.m4a` while it is
/// recorded, then `drop.json` is written last and the folder is renamed to
/// `inbox/<id>/` (the layout `mobile/docs/capture.md` describes).
///
/// The folder and its files are readable after the first unlock
/// (`NSFileProtectionCompleteUntilFirstUserAuthentication`), so a capture
/// started on the lock screen is written without unlocking.
///
/// Never opens the library or calls the core: the app transcribes the drop
/// when it is next open.
final class VoiceDropWriter {
  static let protection = FileProtectionType.completeUntilFirstUserAuthentication

  /// The drop's name: 32 lowercase hex characters.
  let id: String

  private let inbox: URL
  private let partialFolder: URL

  /// Starts a voice drop in `inbox`, making the `.partial` folder.
  init(inbox: URL, id: String? = nil) throws {
    self.inbox = inbox
    let dropId = try id ?? DropWriter.makeId()
    self.id = dropId
    partialFolder = inbox.appendingPathComponent("\(dropId).partial", isDirectory: true)
    try FileManager.default.createDirectory(
      at: partialFolder, withIntermediateDirectories: true,
      attributes: [.protectionKey: VoiceDropWriter.protection])
  }

  /// Where the recorder writes.
  var audioURL: URL { partialFolder.appendingPathComponent(VoiceDropManifest.audioName) }

  /// Writes `drop.json` and renames the folder. Returns the finished folder.
  @discardableResult
  func finish(startedMs: Int64, durationMs: Int64, now: Date = Date()) throws -> URL {
    let manifest = VoiceDropManifest(
      startedMs: startedMs, durationMs: durationMs,
      createdMs: Int64((now.timeIntervalSince1970 * 1000).rounded()))
    try manifest.json().write(
      to: partialFolder.appendingPathComponent("drop.json"),
      options: [.atomic, .completeFileProtectionUntilFirstUserAuthentication])
    try? FileManager.default.setAttributes(
      [.protectionKey: VoiceDropWriter.protection], ofItemAtPath: audioURL.path)
    let finished = inbox.appendingPathComponent(id, isDirectory: true)
    try FileManager.default.moveItem(at: partialFolder, to: finished)
    return finished
  }

  /// Removes the unfinished folder.
  func discard() {
    VoiceDropWriter.removePartial(id: id, in: inbox)
  }

  /// Removes `inbox/<id>.partial`, for a recording whose process died.
  static func removePartial(id: String, in inbox: URL) {
    try? FileManager.default.removeItem(
      at: inbox.appendingPathComponent("\(id).partial", isDirectory: true))
  }
}
