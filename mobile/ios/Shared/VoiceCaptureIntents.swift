// Kanso — VoiceCaptureIntents.swift
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import AppIntents

/// Starts a voice note without opening Kanso, or stops the one in progress
/// (press to start, press again to stop). From the Action Button, Control
/// Center, a widget, Siri or Spotlight.
///
/// It only records: the drop is transcribed when the app is next open.
///
/// Compiled into the app and the widget extension (the controls and widget
/// buttons name it). On iOS 18 `AudioRecordingIntent` makes the system run it
/// in the app, which can hold the microphone.
struct CaptureVoiceIntent: AppIntent {
  static let title: LocalizedStringResource = "Capture Voice Note"
  static let description = IntentDescription(
    "Starts recording a voice note, or stops and saves it. Nothing opens.")
  static let openAppWhenRun = false

  func perform() async throws -> some IntentResult & ProvidesDialog {
    switch VoiceCaptureController.shared.toggle() {
    case .started:
      return .result(dialog: "Recording.")
    case .saved:
      return .result(dialog: "Saved to Inbox.")
    case .tooShort:
      return .result(dialog: "That was too short. Nothing was saved.")
    case .failed(let message):
      return .result(dialog: IntentDialog(stringLiteral: message))
    }
  }
}

// iOS 17 has no `AudioRecordingIntent`; from iOS 18 the system runs this
// intent in the app, keeping the microphone available.
@available(iOS 18.0, *)
extension CaptureVoiceIntent: AudioRecordingIntent {}

/// The Stop button of the Live Activity.
struct StopVoiceCaptureIntent: LiveActivityIntent {
  static let title: LocalizedStringResource = "Stop Voice Note"
  static let openAppWhenRun = false

  func perform() async throws -> some IntentResult {
    _ = VoiceCaptureController.shared.stop()
    return .result()
  }
}
