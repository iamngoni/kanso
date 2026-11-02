// Kanso — VoiceDropTests.swift
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import XCTest

@testable import Runner

/// The voice drop layout and the capture toggle (`mobile/docs/capture.md`).
final class VoiceDropTests: XCTestCase {
  private var root: URL!

  override func setUpWithError() throws {
    root = FileManager.default.temporaryDirectory
      .appendingPathComponent("kanso-voice-tests-\(UUID().uuidString)", isDirectory: true)
    try FileManager.default.createDirectory(at: root, withIntermediateDirectories: true)
  }

  override func tearDownWithError() throws {
    try? FileManager.default.removeItem(at: root)
  }

  private var inbox: URL { root.appendingPathComponent("inbox", isDirectory: true) }

  private func names(in folder: URL) throws -> [String] {
    try FileManager.default.contentsOfDirectory(atPath: folder.path).sorted()
  }

  func testIdIs32LowercaseHexCharacters() throws {
    let writer = try VoiceDropWriter(inbox: inbox)
    XCTAssertNotNil(writer.id.range(of: "^[0-9a-f]{32}$", options: .regularExpression))
  }

  func testRecordingIsInAPartialFolderThenRenamed() throws {
    let writer = try VoiceDropWriter(inbox: inbox)
    XCTAssertEqual(try names(in: inbox), ["\(writer.id).partial"])
    XCTAssertEqual(writer.audioURL.lastPathComponent, "rec.m4a")
    XCTAssertEqual(writer.audioURL.deletingLastPathComponent().lastPathComponent, "\(writer.id).partial")
    try Data("audio".utf8).write(to: writer.audioURL)

    let finished = try writer.finish(startedMs: 1_000, durationMs: 5_000)
    XCTAssertEqual(try names(in: inbox), [writer.id])
    XCTAssertEqual(try names(in: finished), ["drop.json", "rec.m4a"])
  }

  func testManifestFields() throws {
    let writer = try VoiceDropWriter(inbox: inbox)
    let finished = try writer.finish(
      startedMs: 1_700_000_000_000, durationMs: 12_345,
      now: Date(timeIntervalSince1970: 1_700_000_013))
    let data = try Data(contentsOf: finished.appendingPathComponent("drop.json"))
    let json = try XCTUnwrap(JSONSerialization.jsonObject(with: data) as? [String: Any])
    XCTAssertEqual(Set(json.keys), ["version", "kind", "createdAt", "started_ms", "duration_ms", "audio"])
    XCTAssertEqual(json["version"] as? Int, 1)
    XCTAssertEqual(json["kind"] as? String, "voice")
    XCTAssertEqual(json["createdAt"] as? Int64, 1_700_000_013_000)
    XCTAssertEqual(json["started_ms"] as? Int64, 1_700_000_000_000)
    XCTAssertEqual(json["duration_ms"] as? Int64, 12_345)
    XCTAssertEqual(json["audio"] as? String, "rec.m4a")
  }

  func testDiscardRemovesThePartialFolder() throws {
    let writer = try VoiceDropWriter(inbox: inbox)
    writer.discard()
    XCTAssertEqual(try names(in: inbox), [])
  }

  func testRemovePartialOfADeadRecording() throws {
    let writer = try VoiceDropWriter(inbox: inbox)
    VoiceDropWriter.removePartial(id: writer.id, in: inbox)
    XCTAssertEqual(try names(in: inbox), [])
  }

  func testTooShortRecordingsAreDiscarded() {
    XCTAssertFalse(VoiceCaptureRules.shouldKeep(durationMs: 0))
    XCTAssertFalse(VoiceCaptureRules.shouldKeep(durationMs: 999))
    XCTAssertTrue(VoiceCaptureRules.shouldKeep(durationMs: 1_000))
    XCTAssertEqual(VoiceCaptureRules.maxDurationMs, 30 * 60 * 1000)
  }

  func testToggleDecision() {
    let state = VoiceRecordingState(id: String(repeating: "a", count: 32), startedMs: 1)
    XCTAssertEqual(VoiceCaptureRules.decide(state: nil, recorderActive: false), .start)
    XCTAssertEqual(VoiceCaptureRules.decide(state: state, recorderActive: true), .stop)
    XCTAssertEqual(VoiceCaptureRules.decide(state: nil, recorderActive: true), .stop)
    XCTAssertEqual(
      VoiceCaptureRules.decide(state: state, recorderActive: false), .replaceStaleAndStart)
  }

  func testStateRoundTripsThroughJSON() throws {
    let state = VoiceRecordingState(id: "abc", startedMs: 42)
    let data = try JSONEncoder().encode(state)
    XCTAssertEqual(try JSONDecoder().decode(VoiceRecordingState.self, from: data), state)
  }
}
