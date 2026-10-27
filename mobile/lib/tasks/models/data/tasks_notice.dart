// Kanso — tasks_notice.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Why a tick did not happen, for the line that says so.
enum TasksNotice {
  /// The task changed in its note: the list was read again.
  moved,

  /// The note is locked.
  locked,

  /// Anything else.
  failed,
}
