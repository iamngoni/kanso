// Kanso — VoiceDropManifest.swift
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import Foundation

/// The `drop.json` of a voice drop (see "Voice drops" in
/// `mobile/docs/capture.md`).
struct VoiceDropManifest: Equatable {
  /// The audio file's name inside the drop.
  static let audioName = "rec.m4a"

  /// When the recording began, in milliseconds since 1970.
  let startedMs: Int64
  /// How long it ran, in milliseconds.
  let durationMs: Int64
  /// When the drop was finished, in milliseconds since 1970.
  let createdMs: Int64

  /// The manifest as UTF-8 JSON, keys sorted.
  func json() throws -> Data {
    let object: [String: Any] = [
      "version": 1,
      "kind": "voice",
      "createdAt": createdMs,
      "started_ms": startedMs,
      "duration_ms": durationMs,
      "audio": VoiceDropManifest.audioName,
    ]
    return try JSONSerialization.data(withJSONObject: object, options: [.sortedKeys])
  }
}
