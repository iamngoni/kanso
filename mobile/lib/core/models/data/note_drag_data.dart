// Kanso — note_drag_data.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'note_drag_data.freezed.dart';

/// Notes being dragged inside the app: from the list, a tab or a link, onto
/// the tab bar, the editor, a notebook or a tag, or the side of the screen.
///
/// Locked notes travel by id alone: [titles] holds an empty string for a
/// note whose title is sealed, so a drop never shows it.
@freezed
abstract class NoteDragData with _$NoteDragData {
  const factory NoteDragData({
    /// The notes, in the order they are listed; never empty.
    required List<String> noteIds,

    /// Each note's title, in the same order; empty for a locked note.
    required List<String> titles,
  }) = _NoteDragData;
}
