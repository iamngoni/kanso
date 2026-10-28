// Kanso — note_opening.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import '../../../core/bridge/api/types.dart';

part 'note_opening.freezed.dart';

/// A note that was just made and is to be opened: where the app goes.
@freezed
abstract class NoteOpening with _$NoteOpening {
  const factory NoteOpening({
    /// The new note.
    required String noteId,

    /// The place to open it in: the list it was made from, or the notebook
    /// it went into when that list was the trash.
    required ListPlace place,
  }) = _NoteOpening;
}
