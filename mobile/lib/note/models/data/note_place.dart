// Kanso — note_place.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'note_place.freezed.dart';

/// Where the person was in a note when they left its editor: the caret and
/// how far the page was scrolled, so that coming back puts them there.
@freezed
abstract class NotePlace with _$NotePlace {
  const factory NotePlace({
    /// The caret, in UTF-16 units of the note's text.
    required int caret,

    /// The scroll offset of the editor's page.
    required double scroll,
  }) = _NotePlace;
}
