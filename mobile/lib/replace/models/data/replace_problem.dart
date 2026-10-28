// Kanso — replace_problem.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Why the last Replace, or its undo, did not happen.
enum ReplaceProblem {
  /// The text to find cannot be used (a regular expression that does not
  /// parse).
  badPattern,

  /// The notes could not be changed.
  applyFailed,

  /// The notes could not be put back.
  undoFailed,
}
