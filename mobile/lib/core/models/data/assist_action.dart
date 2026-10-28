// Kanso — assist_action.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// What the inline assistant does with the text it is given.
enum AssistAction {
  /// Says the same, better written.
  improve,

  /// Says the same in fewer words.
  shorten,

  /// Corrects spelling and grammar only.
  fix,

  /// Writes what comes next.
  continueWriting,

  /// Sums the text up.
  summarize,
}
