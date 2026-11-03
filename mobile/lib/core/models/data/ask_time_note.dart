// Kanso — ask_time_note.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'ask_time_note.freezed.dart';

/// The time a question was understood to be about, for the range chip.
@freezed
abstract class AskTimeNote with _$AskTimeNote {
  const factory AskTimeNote({
    /// "Written October 2026".
    required String label,

    /// Said when too few notes were written in the range; empty otherwise.
    @Default('') String relaxedLine,
  }) = _AskTimeNote;
}
