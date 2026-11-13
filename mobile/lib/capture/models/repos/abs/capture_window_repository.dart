// Kanso — capture_window_repository.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../data/capture_mode.dart';

/// The window the capture sheet over another app is in (Android's
/// `CaptureActivity`): the mode its link asks for, and closing it so the
/// person is back in the app beneath.
abstract interface class CaptureWindowRepository {
  /// The mode the sheet opens in: Task for `kanso://open/new?task=1` (the
  /// Add task tile and shortcut), else Note (the Capture shortcut).
  Future<CaptureMode> mode();

  /// Closes the window; the app beneath is in front again.
  Future<void> finish();

  /// Opens [link] in Kanso itself and closes the window, for what needs
  /// the app (a new note to write in).
  Future<void> openInKanso(String link);
}
