// Kanso — assist_placement.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Where an accepted suggestion goes.
enum AssistPlacement {
  /// In place of the selected text.
  replace,

  /// After the selected text, or at the caret.
  insertBelow,
}
