// Kanso — VoiceDropWriter.swift
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import Foundation

/// Writes one *voice drop*: `inbox/<id>.partial/` holds `rec.caf` while it is
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

  /// Turns the `<id>.partial` of a recording whose process died into a real
  /// drop, so a capture is never lost. Returns the finished folder, or nil
  /// when the folder has no recording worth keeping (no audio file, an empty
  /// one, or under the minimum length) and it was removed.
  ///
  /// The duration is `startedMs` to the audio file's last write; without
  /// `startedMs` the partial folder's creation time stands in for it.
  @discardableResult
  static func recoverPartial(
    id: String, startedMs: Int64? = nil, in inbox: URL, now: Date = Date()
  ) -> URL? {
    let fm = FileManager.default
    let partial = inbox.appendingPathComponent("\(id).partial", isDirectory: true)
    let audio = partial.appendingPathComponent(VoiceDropManifest.audioName)
    guard let audioAttrs = try? fm.attributesOfItem(atPath: audio.path) else {
      return nil  // Not a voice partial (a share's, maybe): leave it alone.
    }
    let size = (audioAttrs[.size] as? NSNumber)?.int64Value ?? 0
    let endMs = Int64(
      ((audioAttrs[.modificationDate] as? Date) ?? now).timeIntervalSince1970 * 1000)
    let folderAttrs = try? fm.attributesOfItem(atPath: partial.path)
    let started =
      startedMs
      ?? Int64(((folderAttrs?[.creationDate] as? Date) ?? now).timeIntervalSince1970 * 1000)
    let duration = min(max(endMs - started, 0), VoiceCaptureRules.maxDurationMs)
    guard size > 0, VoiceCaptureRules.shouldKeep(durationMs: duration) else {
      removePartial(id: id, in: inbox)
      return nil
    }
    let manifest = VoiceDropManifest(
      startedMs: started, durationMs: duration,
      createdMs: Int64((now.timeIntervalSince1970 * 1000).rounded()))
    let finished = inbox.appendingPathComponent(id, isDirectory: true)
    do {
      try manifest.json().write(
        to: partial.appendingPathComponent("drop.json"),
        options: [.atomic, .completeFileProtectionUntilFirstUserAuthentication])
      try fm.moveItem(at: partial, to: finished)
      return finished
    } catch {
      return nil  // Kept as a partial; the next sweep tries again.
    }
  }

  /// Recovers every voice partial in `inbox` except `activeId` (the recording
  /// in progress). Returns how many became drops.
  @discardableResult
  static func sweepStalePartials(in inbox: URL, activeId: String?, now: Date = Date()) -> Int {
    let names = (try? FileManager.default.contentsOfDirectory(atPath: inbox.path)) ?? []
    var recovered = 0
    for name in names where name.hasSuffix(".partial") {
      let id = String(name.dropLast(".partial".count))
      if id == activeId { continue }
      if recoverPartial(id: id, in: inbox, now: now) != nil { recovered += 1 }
    }
    return recovered
  }
}
