// Kanso — capture_mode.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// What the capture sheet makes of what is typed: a note of its own, or one
/// task (the desktop Quick Capture's Note | Task toggle).
enum CaptureMode { note, task }

/// Where a task typed into the capture sheet has got to.
enum CaptureTaskStatus {
  /// Nothing is being saved; there is nothing to say.
  idle,

  /// Being written.
  saving,

  /// Written; the field is empty again for the next one.
  saved,

  /// The destination, the Journal, the Inbox or the note, is locked.
  locked,

  /// There was no task in the text.
  nothing,

  /// Could not be written; trying again may work.
  failed,
}
