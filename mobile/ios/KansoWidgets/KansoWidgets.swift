// Kanso — KansoWidgets.swift
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import AppIntents
import SwiftUI
import WidgetKit

/// Kanso's links for capture (see `mobile/docs/capture.md`).
private enum KansoLink {
  static let newNote = URL(string: "kanso://open/new")!
  static let voiceNote = URL(string: "kanso://open/new?voice=1")!
  static let today = URL(string: "kanso://open/daily")!
}

/// The widget shows no note, so there is one entry and nothing in it.
struct KansoEntry: TimelineEntry {
  let date: Date
}

/// Hands WidgetKit the one entry. The colors are read when the widget is
/// drawn; the app asks for a reload when it changes them.
struct KansoProvider: TimelineProvider {
  func placeholder(in context: Context) -> KansoEntry {
    KansoEntry(date: .now)
  }

  func getSnapshot(in context: Context, completion: @escaping (KansoEntry) -> Void) {
    completion(KansoEntry(date: .now))
  }

  func getTimeline(in context: Context, completion: @escaping (Timeline<KansoEntry>) -> Void) {
    completion(Timeline(entries: [KansoEntry(date: .now)], policy: .never))
  }
}

/// What the widget draws in each size.
struct KansoWidgetView: View {
  @Environment(\.widgetFamily) private var family
  @Environment(\.colorScheme) private var scheme

  let entry: KansoEntry

  private var palette: WidgetPalette { .current(for: scheme) }

  var body: some View {
    switch family {
    case .systemMedium:
      HStack(spacing: 10) {
        action("New note", symbol: "square.and.pencil", url: KansoLink.newNote)
        voiceAction
        action("Today", symbol: "calendar", url: KansoLink.today)
      }
      .containerBackground(palette.background, for: .widget)

    case .accessoryCircular:
      voiceCircle
        .containerBackground(.clear, for: .widget)

    case .accessoryRectangular:
      HStack(spacing: 8) {
        Image(systemName: "square.and.pencil")
          .font(.title3)
          .widgetAccentable()
        Text("New note")
          .font(.headline)
        Spacer(minLength: 0)
      }
      .widgetURL(KansoLink.newNote)
      .containerBackground(.clear, for: .widget)

    default:
      VStack(alignment: .leading, spacing: 0) {
        Image(systemName: "square.and.pencil")
          .font(.system(size: 28, weight: .regular))
          .foregroundStyle(palette.accent)
        Spacer(minLength: 0)
        Text("New note")
          .font(.headline)
          .foregroundStyle(palette.text)
        Text("Kanso")
          .font(.caption)
          .foregroundStyle(palette.muted)
      }
      .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .leading)
      .widgetURL(KansoLink.newNote)
      .containerBackground(palette.background, for: .widget)
    }
  }

  /// The lock-screen circle: Voice Note. From iOS 18 it records in place (the
  /// intent is an `AudioRecordingIntent`, which the system runs in the app); on
  /// iOS 17 the intent would run in this extension, which can't hold the
  /// microphone, so the circle opens Kanso on the voice note instead.
  @ViewBuilder
  private var voiceCircle: some View {
    let label = ZStack {
      AccessoryWidgetBackground()
      Image(systemName: "mic.fill")
        .font(.title2)
        .widgetAccentable()
    }
    if #available(iOS 18, *) {
      Button(intent: CaptureVoiceIntent()) { label }
        .buttonStyle(.plain)
        .accessibilityLabel("Voice note")
    } else {
      label
        .widgetURL(KansoLink.voiceNote)
        .accessibilityLabel("Voice note")
    }
  }

  /// The medium widget's Voice note button, with the same iOS 18 split.
  @ViewBuilder
  private var voiceAction: some View {
    if #available(iOS 18, *) {
      Button(intent: CaptureVoiceIntent()) { actionLabel("Voice note", symbol: "mic") }
        .buttonStyle(.plain)
    } else {
      action("Voice note", symbol: "mic", url: KansoLink.voiceNote)
    }
  }

  /// One of the medium widget's three buttons.
  private func action(_ title: LocalizedStringKey, symbol: String, url: URL) -> some View {
    Link(destination: url) { actionLabel(title, symbol: symbol) }
  }

  private func actionLabel(_ title: LocalizedStringKey, symbol: String) -> some View {
    VStack(spacing: 8) {
      Image(systemName: symbol)
        .font(.system(size: 26, weight: .regular))
        .foregroundStyle(palette.accent)
      Text(title)
        .font(.footnote.weight(.medium))
        .foregroundStyle(palette.text)
        .lineLimit(1)
        .minimumScaleFactor(0.8)
    }
    .frame(maxWidth: .infinity, maxHeight: .infinity)
    .background(palette.surface, in: RoundedRectangle(cornerRadius: 14, style: .continuous))
  }
}

/// The one widget: a way to start a note.
struct KansoWidget: Widget {
  let kind = "app.kanso.Kanso.widget"

  var body: some WidgetConfiguration {
    StaticConfiguration(kind: kind, provider: KansoProvider()) { entry in
      KansoWidgetView(entry: entry)
    }
    .configurationDisplayName("Kanso")
    .description("Start a note, a voice note or today's note.")
    .supportedFamilies([
      .systemSmall, .systemMedium, .accessoryCircular, .accessoryRectangular,
    ])
  }
}

@main
struct KansoWidgetBundle: WidgetBundle {
  var body: some Widget {
    KansoWidget()
    VoiceCaptureLiveActivity()
    if #available(iOS 18, *) {
      VoiceNoteControl()
      QuickNoteControl()
    }
  }
}
