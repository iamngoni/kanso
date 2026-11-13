// Kanso — schedule_status.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Whether the Schedule has been read.
enum ScheduleStatus {
  /// Being read for the first time.
  loading,

  /// Read; kept up to date as the library changes.
  ready,

  /// The last reading failed.
  failed,
}

/// How the Schedule lays out the month.
enum ScheduleMode {
  /// The month's weeks, with the chosen day's items under them.
  month,

  /// Every day of the month that has anything on it, one after another.
  agenda,
}

/// Why a tick or a move did not happen, for the line that says so.
enum ScheduleNotice {
  /// The item changed in its note: the Schedule was read again.
  moved,

  /// The note is locked.
  locked,

  /// Anything else.
  failed,
}
