// Kanso — preferences.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import '../../bridge/api/types.dart' show SortKey;
import '../../bridge/api/types.dart' show DiagramRenderer;
import 'kanso_fonts.dart';
import 'kanso_icon_set.dart';
import 'kanso_text_size.dart';
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

    /// The size of the text of notes in the editor.
    @JsonKey(unknownEnumValue: KansoTextSize.standard)
    @Default(KansoTextSize.standard)
    KansoTextSize textSize,

    /// The size of the rendered preview's text.
    @JsonKey(unknownEnumValue: KansoTextSize.standard)
    @Default(KansoTextSize.standard)
    KansoTextSize previewTextSize,

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
  }) = _Preferences;

  factory Preferences.fromJson(Map<String, dynamic> json) =>
      _$PreferencesFromJson(json);
}
