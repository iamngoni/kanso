// Kanso — replace_option.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// A toggle of Replace in all notes.
enum ReplaceOption {
  /// Match upper and lower case as typed.
  matchCase,

  /// Only whole words match.
  wholeWord,

  /// The text to find is a regular expression (`$1` in the replacement
  /// stands for a group).
  regex,

  /// Leave matches inside fenced code alone.
  skipCode,

  /// Look in the notes in the Trash too.
  includeTrash,
}
