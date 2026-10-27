// Kanso — history_problem.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Why restoring a version, or reading what changed since it, did not
/// happen.
enum HistoryProblem {
  /// The note changed while its history was open: the history was read
  /// again and the version can be chosen again.
  changedMeanwhile,

  /// The version is no longer kept.
  versionGone,

  /// Restoring did not go through.
  restoreFailed,

  /// What changed since the version could not be worked out.
  diffFailed,
}
