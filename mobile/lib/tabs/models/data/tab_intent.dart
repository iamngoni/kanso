// Kanso — tab_intent.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

/// What a keyboard shortcut (or a menu) asks of the tabs on a tablet. The
/// `TabActions` widget turns each into an event of the tabs' BLoC; the key
/// map binds them (the desktop's keys are in its `tabs.rs`).
sealed class TabIntent extends Intent {
  const TabIntent();
}

/// Close the showing tab (⌘W). With one tab open nothing happens: unlike the
/// desktop, where the last tab closes the window, a tablet has no window of
/// the app to close, and a note is closed by trashing it.
class CloseTabIntent extends TabIntent {
  const CloseTabIntent();
}

/// Show the next tab (Ctrl-Tab), wrapping past the last.
class NextTabIntent extends TabIntent {
  const NextTabIntent();
}

/// Show the tab before (Ctrl-Shift-Tab), wrapping from the first.
class PreviousTabIntent extends TabIntent {
  const PreviousTabIntent();
}

/// Bring back the tab closed last (⌘⇧T).
class ReopenClosedTabIntent extends TabIntent {
  const ReopenClosedTabIntent();
}

/// Show tab [number] counted from 1 (⌘⇧1 to ⌘⇧9); 9 is the last.
class GoToTabIntent extends TabIntent {
  const GoToTabIntent(this.number);

  final int number;
}

/// Open note [noteId] in a new tab (⌘↩ on a note of the list, ⌘-click).
class OpenInNewTabIntent extends TabIntent {
  const OpenInNewTabIntent(this.noteId);

  final String noteId;
}
