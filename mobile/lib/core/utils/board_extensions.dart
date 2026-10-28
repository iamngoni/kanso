// Kanso — board_extensions.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../bridge/api/types.dart';

/// What a board and its cards tell without the bridge's help.
extension BoardViewCount on BoardView {
  /// How many cards there are in all.
  int get cardCount =>
      columns.fold(0, (total, column) => total + column.cards.length);
}

extension CardKeyTarget on CardKey {
  /// The note the card opens.
  String get noteIdOf => switch (this) {
    CardKey_Note(:final id) => id,
    CardKey_Task(:final noteId) => noteId,
  };

  /// The line a task card opens at; null for a note.
  int? get lineOf => switch (this) {
    CardKey_Note() => null,
    CardKey_Task(:final line) => line,
  };
}
