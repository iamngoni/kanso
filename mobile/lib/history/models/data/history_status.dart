// Kanso — history_status.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// How far a note's history has been read.
enum HistoryStatus {
  /// Being read.
  loading,

  /// Read and shown (a note with no versions yet is ready, and empty).
  ready,

  /// The note is locked and closed: its versions are sealed and not read.
  locked,

  /// There is no such note any more.
  notFound,

  /// Could not be read; the screen says so and offers to try again.
  failed,
}
