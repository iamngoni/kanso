// Kanso: import_stage.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Where the import flow is.
enum ImportStage {
  /// Choosing where the notes come from.
  sources,

  /// Reading the export to say what it holds; nothing is written.
  reading,

  /// What is in the export, before anything is written.
  summary,

  /// Writing the notes.
  importing,

  /// What the import did.
  done,

  /// Something went wrong that the person chooses how to leave.
  failed,
}
