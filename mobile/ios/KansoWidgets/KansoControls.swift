// Kanso — KansoControls.swift
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import AppIntents
import SwiftUI
import WidgetKit

/// Control Center, lock-screen and Action Button control: starts or stops a
/// voice note without opening Kanso.
@available(iOS 18.0, *)
struct VoiceNoteControl: ControlWidget {
  var body: some ControlWidgetConfiguration {
    StaticControlConfiguration(kind: "app.kanso.Kanso.control.voice") {
      ControlWidgetButton(action: CaptureVoiceIntent()) {
        Label("Voice Note", systemImage: "mic.fill")
      }
    }
    .displayName("Voice Note")
    .description("Start or stop a voice note.")
  }
}

/// Control that opens Kanso on the capture sheet.
@available(iOS 18.0, *)
struct QuickNoteControl: ControlWidget {
  var body: some ControlWidgetConfiguration {
    StaticControlConfiguration(kind: "app.kanso.Kanso.control.quick") {
      ControlWidgetButton(action: OpenURLIntent(URL(string: "kanso://open/new")!)) {
        Label("Quick Note", systemImage: "pencil")
      }
    }
    .displayName("Quick Note")
    .description("Open Kanso on a new note.")
  }
}
