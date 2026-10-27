// Kanso — preferences.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import '../../bridge/api/types.dart' show SortKey;
import 'kanso_fonts.dart';
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

    /// The size of the text of notes.
    @JsonKey(unknownEnumValue: KansoTextSize.standard)
    @Default(KansoTextSize.standard)
    KansoTextSize textSize,

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

    /// The notebook new notes go in, unless they are made from a notebook's
    /// own list. Null, or one that is gone or locked, is the first notebook.
    String? defaultNotebookId,
  }) = _Preferences;

  factory Preferences.fromJson(Map<String, dynamic> json) =>
      _$PreferencesFromJson(json);
}
