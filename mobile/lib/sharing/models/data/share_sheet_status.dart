// Kanso — share_sheet_status.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// What the share sheet of a note has to show.
enum ShareSheetStatus {
  /// Reading the note's sharing.
  loading,

  /// The note is sharing, or can be: people, links and the invite field.
  ready,

  /// The note is locked (or in a locked notebook): "Locked notes can't be
  /// shared." and nothing else.
  locked,

  /// The note can't be shared for a reason the library words (it is in the
  /// trash).
  notShareable,

  /// Sharing needs an account.
  signedOut,

  /// The sharing could not be read; the sheet offers to try again.
  failed,
}
