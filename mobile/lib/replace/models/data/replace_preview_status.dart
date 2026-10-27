// Kanso — replace_preview_status.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Where the preview of Replace in all notes is.
enum ReplacePreviewStatus {
  /// There is no text to find, so nothing is previewed.
  idle,

  /// Looking.
  loading,

  /// Found (maybe nothing), or the pattern's own problem is in the preview.
  ready,

  /// The notes could not be searched.
  failed,
}
