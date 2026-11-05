// Kanso — preferences.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import '../../bridge/api/types.dart' show SortKey;
import '../../bridge/api/types.dart' show DiagramRenderer;
import 'kanso_fonts.dart';
import 'kanso_icon_set.dart';
import 'kanso_text_size.dart';
import 'review_prompt_record.dart';
import 'theme_selection.dart';

part 'preferences.freezed.dart';
part 'preferences.g.dart';

/// What the person has chosen that the phone remembers: how the app looks,
/// how lists are sorted, and which notices are done with.
///
/// They are kept in `preferences.json` in the library folder, so they stay on
/// this phone and out of backups. A value the file lacks, or that a newer
/// version wrote and this one does not know, is the default.
@freezed
abstract class Preferences with _$Preferences {
  const factory Preferences({
    /// The themes the interface is drawn in.
    @Default(ThemeSelection()) ThemeSelection themeSelection,

    /// The fonts of notes.
    @Default(KansoFonts()) KansoFonts fonts,

    /// The size of the text of notes in the editor, from 0.8 to 1.8.
    @JsonKey(
      fromJson: textScaleFromJson,
      toJson: textScaleToJson,
      defaultValue: 1.0,
    )
    @Default(1.0)
    double textSize,

    /// The size of the rendered preview's text, from 0.8 to 1.8.
    @JsonKey(
      fromJson: textScaleFromJson,
      toJson: textScaleToJson,
      defaultValue: 1.0,
    )
    @Default(1.0)
    double previewTextSize,

    /// The size of the interface's labels and text, from 0.9 to 1.3.
    @JsonKey(
      fromJson: interfaceTextScaleFromJson,
      toJson: interfaceTextScaleToJson,
      defaultValue: 1.0,
    )
    @Default(1.0)
    double interfaceTextSize,

    /// The icon theme the app is drawn in, by its ID. A theme this version
    /// does not know is Kanso's own icons.
    @Default(KansoIconSet.lucideId) String iconTheme,

    /// What draws Mermaid diagrams.
    @JsonKey(unknownEnumValue: DiagramRenderer.pele)
    @Default(DiagramRenderer.pele)
    DiagramRenderer diagramRenderer,

    /// What note lists are sorted by.
    @JsonKey(unknownEnumValue: SortKey.updated)
    @Default(SortKey.updated)
    SortKey sort,

    /// Whether lists run from the largest to the smallest sort value (the
    /// newest first, Z to A).
    @Default(true) bool descending,

    /// Whether the notice that this phone's notes are not in device backups
    /// has been shown.
    @Default(false) bool backupNoticeSeen,

    /// Whether the offer to unlock locked notes with a biometric, made once
    /// after the first password unlock, has been shown.
    @Default(false) bool biometricOfferSeen,

    /// The notebook new notes go in, unless they are made from a notebook's
    /// own list. Null, or one that is gone or locked, is the first notebook.
    String? defaultNotebookId,

    /// The notes open as tabs on a tablet, left to right (the desktop keeps
    /// them in its settings too). Notes that are gone by the next start are
    /// dropped then. Empty on a phone, which has no tabs.
    @Default(<String>[]) List<String> openTabs,

    /// The index in [openTabs] of the tab that was showing.
    @Default(0) int activeTab,

    /// Whether pictures attached to notes are read on this device for search.
    @Default(false) bool searchImageText,

    /// Whether this phone may show reminders for tasks with a due time.
    /// Notification permission is asked separately by the operating system.
    @Default(true) bool remindersEnabled,

    /// Whether the open note is shown by itself, without the library, the
    /// list, tabs or the note's bars.
    @Default(false) bool focusMode,

    /// Whether focus mode shows the open note in Live, and leaving it goes
    /// back to the mode the note was in.
    @Default(false) bool livePreviewInFocus,

    /// Whether the editor offers past notes about the paragraph being
    /// written, in a pill above the keyboard (when related notes are on).
    @Default(true) bool resurface,

    /// How close a past note must be to be offered: "closer", "default" or
    /// "looser". A value this version does not know is "default".
    @Default('default') String resurfaceCloseness,

    /// Whether a daily note lists the notes written on its date in earlier
    /// years.
    @Default(true) bool onThisDay,

    /// What decides whether to ask for a review of the app: when it was
    /// first opened, how often it is used, and when it last asked.
    @Default(ReviewPromptRecord()) ReviewPromptRecord reviewPrompt,

    /// The PDFs whose pages are shown as printed rather than smart-inverted
    /// in a dark theme, keyed by attachment ID.
    @Default(<String, bool>{}) Map<String, bool> pdfOriginalPages,
  }) = _Preferences;

  factory Preferences.fromJson(Map<String, dynamic> json) =>
      _$PreferencesFromJson(json);
}
