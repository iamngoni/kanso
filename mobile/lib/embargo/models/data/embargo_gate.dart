// Kanso — embargo_gate.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// What an embargo holds back until it lifts.
enum EmbargoGate {
  /// Publishing the note at a link anyone can open.
  publish,

  /// Making an invite link to the shared note.
  shareLink,

  /// Exporting the note as a web page or a PDF.
  export,
}
