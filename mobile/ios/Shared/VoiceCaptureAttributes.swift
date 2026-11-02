// Kanso — VoiceCaptureAttributes.swift
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import ActivityKit
import Foundation

/// The Live Activity of a voice capture, shared by the app (which starts and
/// ends it) and the widget extension (which draws it).
struct VoiceCaptureAttributes: ActivityAttributes {
  struct ContentState: Codable, Hashable {
    /// `true` once the recording is saved: the activity then shows
    /// "Saved to Inbox" until the system dismisses it.
    var saved: Bool
  }

  /// When the recording began; the timer counts from here.
  let startDate: Date
}
