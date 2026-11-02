// Kanso — VoiceCaptureLiveActivity.swift
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import ActivityKit
import AppIntents
import SwiftUI
import WidgetKit

/// The lock-screen banner and Dynamic Island of a voice capture: a red dot,
/// the time so far and Stop. After the save it reads "Saved to Inbox" until
/// the system dismisses it.
struct VoiceCaptureLiveActivity: Widget {
  var body: some WidgetConfiguration {
    ActivityConfiguration(for: VoiceCaptureAttributes.self) { context in
      VoiceCaptureBanner(context: context)
        .padding(.horizontal, 16)
        .padding(.vertical, 12)
        .activityBackgroundTint(Color.black.opacity(0.6))
    } dynamicIsland: { context in
      DynamicIsland {
        DynamicIslandExpandedRegion(.leading) {
          RecordingDot(saved: context.state.saved)
        }
        DynamicIslandExpandedRegion(.center) {
          VoiceCaptureTime(context: context).font(.title2.monospacedDigit())
        }
        DynamicIslandExpandedRegion(.trailing) {
          if !context.state.saved { StopButton() }
        }
      } compactLeading: {
        RecordingDot(saved: context.state.saved)
      } compactTrailing: {
        if context.state.saved {
          Image(systemName: "checkmark")
        } else {
          Text(timerInterval: context.attributes.startDate...Date.distantFuture, countsDown: false)
            .monospacedDigit()
            .frame(width: 44)
        }
      } minimal: {
        RecordingDot(saved: context.state.saved)
      }
    }
  }
}

private struct VoiceCaptureBanner: View {
  let context: ActivityViewContext<VoiceCaptureAttributes>

  var body: some View {
    HStack(spacing: 12) {
      RecordingDot(saved: context.state.saved)
      VoiceCaptureTime(context: context)
        .font(.title3.monospacedDigit())
        .foregroundStyle(.white)
      Spacer(minLength: 0)
      if !context.state.saved { StopButton() }
    }
  }
}

/// The elapsed time, or "Saved to Inbox" once the recording is saved.
private struct VoiceCaptureTime: View {
  let context: ActivityViewContext<VoiceCaptureAttributes>

  var body: some View {
    if context.state.saved {
      Text("Saved to Inbox")
    } else {
      Text(timerInterval: context.attributes.startDate...Date.distantFuture, countsDown: false)
    }
  }
}

private struct RecordingDot: View {
  let saved: Bool

  var body: some View {
    if saved {
      Image(systemName: "checkmark.circle.fill").foregroundStyle(.green)
    } else {
      Circle().fill(.red).frame(width: 12, height: 12)
        .accessibilityLabel("Recording")
    }
  }
}

private struct StopButton: View {
  var body: some View {
    Button(intent: StopVoiceCaptureIntent()) {
      Label("Stop", systemImage: "stop.fill")
        .font(.subheadline.weight(.semibold))
    }
    .buttonStyle(.borderedProminent)
    .tint(.red)
  }
}
