// Kanso — tasks_status.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Whether the tasks have been read.
enum TasksStatus {
  /// Being read for the first time.
  loading,

  /// Read; kept up to date as the library changes.
  ready,

  /// The last reading failed.
  failed,
}
