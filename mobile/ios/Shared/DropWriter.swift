// Kanso — DropWriter.swift
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import Foundation
import Security
import UniformTypeIdentifiers

/// What can go wrong while a drop is written.
enum DropError: Error {
  /// The drop already holds `DropWriter.maxFiles` files.
  case tooManyFiles
  /// The system gave no random bytes for the drop's name.
  case noRandomBytes
  /// The drop was finished or discarded already.
  case closed
}

/// Writes one *drop*: a folder in the library's `inbox/` that holds what was
/// shared (text, a link, files) for the library to save as a note.
///
/// The layout is the one `mobile/docs/capture.md` describes:
///
/// 1. `inbox/<id>.partial/` is made, `<id>` being 32 lowercase hex characters.
/// 2. Each file is copied in as `0.<ext>`, `1.<ext>`…
/// 3. `drop.json` is written last.
/// 4. The folder is renamed to `inbox/<id>/`.
///
/// Files are copied by URL, so a large image or PDF is never held in memory.
/// The share extension's providers hand out a file that is deleted when their
/// callback returns, so `addFile` is called inside that callback.
///
/// Safe to call from any thread.
final class DropWriter {
  /// The most files one drop may hold.
  static let maxFiles = 20

  /// The drop's name.
  let id: String

  private let inbox: URL
  private let partialFolder: URL
  private let lock = NSLock()
  private var files: [[String: String]] = []
  private var closed = false

  /// Starts a drop in `inbox`, making the folder (and `inbox`) if needed.
  init(inbox: URL) throws {
    self.inbox = inbox
    id = try DropWriter.makeId()
    partialFolder = inbox.appendingPathComponent("\(id).partial", isDirectory: true)
    try FileManager.default.createDirectory(
      at: partialFolder, withIntermediateDirectories: true)
  }

  /// How many files the drop holds so far.
  var fileCount: Int {
    lock.lock()
    defer { lock.unlock() }
    return files.count
  }

  /// Copies `source` into the drop.
  ///
  /// - Parameters:
  ///   - source: the file to copy.
  ///   - name: the name to show for it; the source's own name when `nil`.
  ///   - type: its MIME type, as the sharing app said; guessed from the
  ///     extension when `nil`.
  /// - Throws: `DropError.tooManyFiles` past the 20th file, or the copy's error.
  func addFile(from source: URL, name: String? = nil, type: String? = nil) throws {
    lock.lock()
    defer { lock.unlock() }
    guard !closed else { throw DropError.closed }
    guard files.count < DropWriter.maxFiles else { throw DropError.tooManyFiles }

    let shownName = (name?.isEmpty == false ? name : nil) ?? source.lastPathComponent
    let fileExtension = DropWriter.fileExtension(
      named: source.lastPathComponent, fallbackName: shownName, type: type)
    let path = "\(files.count).\(fileExtension)"
    try FileManager.default.copyItem(
      at: source, to: partialFolder.appendingPathComponent(path))

    var entry = ["path": path, "name": shownName]
    if let mime = type ?? DropWriter.mimeType(forExtension: fileExtension) {
      entry["type"] = mime
    }
    files.append(entry)
  }

  /// Writes `drop.json` and renames the folder, which is what makes the drop
  /// readable. Returns the finished folder.
  ///
  /// Empty strings are left out of `drop.json`.
  @discardableResult
  func finish(subject: String? = nil, text: String? = nil, url: String? = nil) throws -> URL {
    lock.lock()
    defer { lock.unlock() }
    guard !closed else { throw DropError.closed }

    var json: [String: Any] = [
      "version": 1,
      "createdAt": Int64((Date().timeIntervalSince1970 * 1000).rounded()),
    ]
    for (key, value) in [("subject", subject), ("text", text), ("url", url)] {
      if let value, !value.isEmpty { json[key] = value }
    }
    if !files.isEmpty { json["files"] = files }

    let data = try JSONSerialization.data(withJSONObject: json, options: [.sortedKeys])
    try data.write(to: partialFolder.appendingPathComponent("drop.json"), options: .atomic)

    let finished = inbox.appendingPathComponent(id, isDirectory: true)
    try FileManager.default.moveItem(at: partialFolder, to: finished)
    closed = true
    return finished
  }

  /// Writes a drop of text and a link alone, with no files.
  @discardableResult
  static func write(
    in inbox: URL, subject: String? = nil, text: String? = nil, url: String? = nil
  ) throws -> URL {
    let writer = try DropWriter(inbox: inbox)
    do {
      return try writer.finish(subject: subject, text: text, url: url)
    } catch {
      writer.discard()
      throw error
    }
  }

  /// Removes the unfinished folder. Does nothing once the drop is finished.
  func discard() {
    lock.lock()
    defer { lock.unlock() }
    guard !closed else { return }
    closed = true
    try? FileManager.default.removeItem(at: partialFolder)
  }

  /// 32 lowercase hex characters from the system's random source.
  static func makeId() throws -> String {
    var bytes = [UInt8](repeating: 0, count: 16)
    guard SecRandomCopyBytes(kSecRandomDefault, bytes.count, &bytes) == errSecSuccess
    else { throw DropError.noRandomBytes }
    return bytes.map { String(format: "%02x", $0) }.joined()
  }

  /// The extension a file is saved under: the source's own, then the shown
  /// name's, then the MIME type's, then `bin`. Lowercase letters and digits
  /// only, so the name can't hold a path.
  static func fileExtension(named name: String, fallbackName: String, type: String?) -> String {
    let candidates: [String] = [
      (name as NSString).pathExtension,
      (fallbackName as NSString).pathExtension,
      type.flatMap { UTType(mimeType: $0)?.preferredFilenameExtension } ?? "",
    ]
    for candidate in candidates {
      let clean = String(candidate.lowercased().filter { $0.isASCII && ($0.isLetter || $0.isNumber) })
      if !clean.isEmpty && clean.count <= 8 { return clean }
    }
    return "bin"
  }

  /// The MIME type of a file extension, if the system knows it.
  static func mimeType(forExtension fileExtension: String) -> String? {
    UTType(filenameExtension: fileExtension)?.preferredMIMEType
  }
}
