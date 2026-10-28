// Kanso — window_intent.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

/// What a keyboard shortcut or a menu asks of the windows of the app. The
/// keys are bound by the shortcut catalog (⌘W in a note window, ⌘⇧O).
sealed class WindowIntent extends Intent {
  const WindowIntent();
}

/// Close this window, when it is a note's own window (⌘W, context
/// `NoteWindow`). The main window is never closed.
class CloseNoteWindowIntent extends WindowIntent {
  const CloseNoteWindowIntent();
}

/// Open a note in a window of its own: [noteId], or, when it is null, the
/// note the surface that handles the intent shows.
class OpenInNewWindowIntent extends WindowIntent {
  const OpenInNewWindowIntent([this.noteId]);

  final String? noteId;
}
