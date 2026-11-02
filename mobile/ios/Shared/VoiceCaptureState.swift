// Kanso — VoiceCaptureState.swift
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import Foundation

/// The recording in progress, kept in the App Group so a second run of the
/// intent (a new process, maybe) can find it.
struct VoiceRecordingState: Codable, Equatable {
  /// The drop's id: the `<id>.partial` folder's name.
  let id: String
  /// When the recording began, in milliseconds since 1970.
  let startedMs: Int64
}

/// What one run of the capture intent does.
enum VoiceToggleAction: Equatable {
  /// Nothing is recording: start.
  case start
  /// A recording runs in this process: stop and save it.
  case stop
  /// The state names a recording no live recorder has (the process was
  /// killed): drop the leftovers, then start.
  case replaceStaleAndStart
}

/// The rules of the press-to-start, press-again-to-stop toggle, apart from
/// the recorder so they can be tested.
enum VoiceCaptureRules {
  /// A recording stops by itself after this long.
  static let maxDurationMs: Int64 = 30 * 60 * 1000
  /// A recording shorter than this is thrown away.
  static let minDurationMs: Int64 = 1000

  /// What a run should do.
  ///
  /// - Parameters:
  ///   - state: the saved state of the recording in progress, if any.
  ///   - recorderActive: whether this process has a recorder that is recording.
  static func decide(state: VoiceRecordingState?, recorderActive: Bool) -> VoiceToggleAction {
    if recorderActive { return .stop }
    return state == nil ? .start : .replaceStaleAndStart
  }

  /// Whether a recording of this length is kept.
  static func shouldKeep(durationMs: Int64) -> Bool {
    durationMs >= minDurationMs
  }
}
