// Kanso — capture_over_app_status.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Where saving what was picked in the capture sheet over another app has
/// got to: photos, a photo taken or a PDF, saved as a note in the Inbox.
enum CaptureOverAppStatus {
  /// Nothing picked yet: the sheet is open, or was closed with nothing.
  idle,

  /// The picker is open, or what was picked is being saved.
  working,

  /// Nothing was picked: the picker was closed.
  cancelled,

  /// Saved as a note in the Inbox.
  saved,

  /// The Inbox is locked, and nothing was written.
  locked,

  /// The picker or the save failed.
  failed;

  /// Whether the sheet has nothing more to do, so it can go.
  bool get finished => this != idle && this != working;
}
