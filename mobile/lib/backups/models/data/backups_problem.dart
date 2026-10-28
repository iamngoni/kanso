// Kanso — backups_problem.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Why the last thing done with the backups did not happen.
enum BackupsProblem {
  /// The daily backup, made when the app opened, could not be made.
  dailyFailed,

  /// Back up now could not make its copy.
  backUpFailed,

  /// The backup could not be deleted (it is gone, or it is not one made on
  /// request).
  deleteFailed,

  /// The list could not be read again; the one shown is the last that was.
  listFailed,
}
