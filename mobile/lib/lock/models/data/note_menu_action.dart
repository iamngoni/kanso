// Kanso — note_menu_action.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// What a note's menu offers.
enum NoteMenuAction {
  /// Writes the note as a PDF and offers it to share.
  exportPdf,

  /// Writes the note as one web page file and offers it to share.
  exportHtml,

  /// Shows the note full screen, as slides.
  present,
  lockNote,
  removeLock,
  lockNow,

  /// Shows the note in a window of its own, where the device has more than
  /// one window.
  openInNewWindow,

  /// What the note's bar would offer, had it room for it: in the menu when
  /// the bar gave way (see `KansoTopBar.hiddenOf`).
  info,
  comments,
  share,
  previousDay,
  nextDay,
  calendar,
}
