// Kanso — palette_status.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Where the command palette's results are.
enum PaletteStatus {
  /// Nothing has been asked for yet.
  idle,

  /// The commands and places are being read.
  loading,

  /// The last search's sections hold what matches the query.
  ready,

  /// The search failed; the state's problem says why.
  failed,
}
