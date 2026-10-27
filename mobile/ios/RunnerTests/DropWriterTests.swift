// Kanso — DropWriterTests.swift
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import XCTest

@testable import Runner

/// The drop layout `mobile/docs/capture.md` describes.
final class DropWriterTests: XCTestCase {
  private var root: URL!

  override func setUpWithError() throws {
    root = FileManager.default.temporaryDirectory
      .appendingPathComponent("kanso-drop-tests-\(UUID().uuidString)", isDirectory: true)
    try FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)
  }

  override func tearDownWithError() throws {
    try? FileManager.default.removeItem(at: root)
  }

  private var inbox: URL { root.appendingPathComponent("inbox", isDirectory: true) }

  private func sourceFile(_ name: String, contents: String = "data") throws -> URL {
    let url = root.appendingPathComponent(name)
    try Data(contents.utf8).write(to: url)
    return url
  }

  private func names(in folder: URL) throws -> [String] {
    try FileManager.default.contentsOfDirectory(atPath: folder.path).sorted()
  }

  private func readJSON(_ url: URL) throws -> [String: Any] {
    let data = try Data(contentsOf: url)
    return try XCTUnwrap(JSONSerialization.jsonObject(with: data) as? [String: Any])
  }

  func testIdsAre32LowercaseHexCharactersAndDiffer() throws {
    let pattern = try NSRegularExpression(pattern: "^[0-9a-f]{32}$")
    var seen = Set<String>()
    for _ in 0..<50 {
      let id = try DropWriter.makeId()
      XCTAssertEqual(pattern.numberOfMatches(in: id, range: NSRange(id.startIndex..., in: id)), 1)
      seen.insert(id)
    }
    XCTAssertEqual(seen.count, 50)
  }

  func testUnfinishedDropIsPartialAndFinishRenamesIt() throws {
    let writer = try DropWriter(inbox: inbox)
    XCTAssertEqual(try names(in: inbox), ["\(writer.id).partial"])

    let finished = try writer.finish(text: "hello")
    XCTAssertEqual(finished.lastPathComponent, writer.id)
    XCTAssertEqual(try names(in: inbox), [writer.id])
    XCTAssertTrue(
      FileManager.default.fileExists(atPath: finished.appendingPathComponent("drop.json").path))
  }

  func testDropJsonFields() throws {
    let before = Int64(Date().timeIntervalSince1970 * 1000)
    let writer = try DropWriter(inbox: inbox)
    try writer.addFile(from: try sourceFile("IMG_0042.PNG"), name: "IMG_0042.PNG", type: "image/png")
    try writer.addFile(from: try sourceFile("Quarterly report.pdf"), type: "application/pdf")
    let folder = try writer.finish(
      subject: "A page", text: "Shared text", url: "https://example.com/a")

    XCTAssertEqual(
      try names(in: folder), ["0.png", "1.pdf", "drop.json"])
    let json = try readJSON(folder.appendingPathComponent("drop.json"))
    XCTAssertEqual(json["version"] as? Int, 1)
    XCTAssertGreaterThanOrEqual(try XCTUnwrap(json["createdAt"] as? Int64), before)
    XCTAssertEqual(json["subject"] as? String, "A page")
    XCTAssertEqual(json["text"] as? String, "Shared text")
    XCTAssertEqual(json["url"] as? String, "https://example.com/a")
    let files = try XCTUnwrap(json["files"] as? [[String: String]])
    XCTAssertEqual(
      files,
      [
        ["path": "0.png", "name": "IMG_0042.PNG", "type": "image/png"],
        ["path": "1.pdf", "name": "Quarterly report.pdf", "type": "application/pdf"],
      ])
  }

  func testEmptyFieldsAreLeftOut() throws {
    let folder = try DropWriter.write(in: inbox, subject: "", text: nil, url: "https://a.example")
    let json = try readJSON(folder.appendingPathComponent("drop.json"))
    XCTAssertEqual(Set(json.keys), ["version", "createdAt", "url"])
  }

  func testTwentyFileCap() throws {
    let writer = try DropWriter(inbox: inbox)
    let source = try sourceFile("a.png")
    for _ in 0..<DropWriter.maxFiles {
      try writer.addFile(from: source, type: "image/png")
    }
    XCTAssertEqual(writer.fileCount, 20)
    XCTAssertThrowsError(try writer.addFile(from: source, type: "image/png")) { error in
      guard case DropError.tooManyFiles = error else { return XCTFail("\(error)") }
    }
    let folder = try writer.finish()
    let files = try XCTUnwrap(readJSON(folder.appendingPathComponent("drop.json"))["files"] as? [Any])
    XCTAssertEqual(files.count, 20)
    XCTAssertTrue(FileManager.default.fileExists(atPath: folder.appendingPathComponent("19.png").path))
    XCTAssertFalse(FileManager.default.fileExists(atPath: folder.appendingPathComponent("20.png").path))
  }

  func testDiscardRemovesThePartialFolder() throws {
    let writer = try DropWriter(inbox: inbox)
    try writer.addFile(from: try sourceFile("a.png"), type: "image/png")
    writer.discard()
    XCTAssertEqual(try names(in: inbox), [])
    XCTAssertThrowsError(try writer.finish())
  }

  func testExtensionsCannotHoldAPath() {
    XCTAssertEqual(DropWriter.fileExtension(named: "x.PnG", fallbackName: "y", type: nil), "png")
    XCTAssertEqual(DropWriter.fileExtension(named: "x", fallbackName: "y", type: "application/pdf"), "pdf")
    XCTAssertEqual(DropWriter.fileExtension(named: "x", fallbackName: "y", type: nil), "bin")
  }
}
