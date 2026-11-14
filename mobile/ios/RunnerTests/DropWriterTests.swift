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

  // #390 slice 18: "Add Kanso task" leaves a task drop.
  func testATaskDropCarriesItsLineDayAndTarget() throws {
    let line = "- [ ] film Pixel review @2026-11-13T15:00 every week"
    let folder = try DropWriter.writeTask(
      in: inbox, line: line, day: "2026-11-12", time: "09:41", target: "inbox")
    XCTAssertEqual(try names(in: folder), ["drop.json"])
    let json = try readJSON(folder.appendingPathComponent("drop.json"))
    XCTAssertEqual(json["version"] as? Int, 1)
    XCTAssertEqual(json["kind"] as? String, "task")
    XCTAssertEqual(json["line"] as? String, line)
    XCTAssertEqual(json["day"] as? String, "2026-11-12")
    XCTAssertEqual(json["time"] as? String, "09:41")
    XCTAssertEqual(json["target"] as? String, "inbox")
    XCTAssertNil(json["files"])
    XCTAssertNotNil(json["createdAt"] as? Int64)
  }

  func testIntentHelpers() throws {
    var calendar = Calendar(identifier: .gregorian)
    calendar.timeZone = try XCTUnwrap(TimeZone(identifier: "Europe/London"))
    let date = try XCTUnwrap(
      calendar.date(from: DateComponents(year: 2026, month: 3, day: 4, hour: 7, minute: 5)))
    let (day, time) = KansoIntentCore.localDayAndTime(date, calendar: calendar)
    XCTAssertEqual(day, "2026-03-04")
    XCTAssertEqual(time, "07:05")

    XCTAssertTrue(KansoIntentCore.monthFirst(region: "US"))
    XCTAssertFalse(KansoIntentCore.monthFirst(region: "GB"))
    XCTAssertFalse(KansoIntentCore.monthFirst(region: nil))

    XCTAssertEqual(KansoIntentCore.taskTarget(from: nil), "dailyNote")
    XCTAssertEqual(
      KansoIntentCore.taskTarget(from: Data(#"{"captureTaskTarget":"inbox"}"#.utf8)), "inbox")
    XCTAssertEqual(
      KansoIntentCore.taskTarget(from: Data(#"{"captureTaskTarget":"desk"}"#.utf8)), "dailyNote")

    XCTAssertEqual(
      KansoIntentCore.searchLink(" film pixel ")?.absoluteString,
      "kanso://open/search?q=film%20pixel")
    // Foundation may write the colon as `%3A` (iOS 27 does); the app reads
    // the decoded path either way, so compare that.
    let link = try XCTUnwrap(KansoIntentCore.noteLink("note:abc"))
    XCTAssertEqual(link.scheme, "kanso")
    XCTAssertEqual(link.host, "open")
    XCTAssertEqual(link.pathComponents, ["/", "note", "note:abc"])
    XCTAssertNil(KansoIntentCore.noteLink("a/b"))

    XCTAssertEqual(KansoIntentCore.reading(of: nil), .unavailable)
    XCTAssertEqual(KansoIntentCore.reading(of: Data("null".utf8)), .noTask)
    let answer = Data(
      #"{"line":"- [ ] a","words":"a","when":null,"confirmation":"Added: a"}"#.utf8)
    XCTAssertEqual(
      KansoIntentCore.reading(of: answer),
      .task(TaskPreview(line: "- [ ] a", words: "a", when: nil, confirmation: "Added: a")))

    let notes = KansoIntentCore.decodeNotes(
      Data(#"{"notes":[{"id":"note:1","title":"Pixel","notebook":"Inbox"}]}"#.utf8))
    XCTAssertEqual(notes.map(\.id), ["note:1"])
    XCTAssertTrue(KansoIntentCore.decodeNotes(Data("{}".utf8)).isEmpty)
  }
}
