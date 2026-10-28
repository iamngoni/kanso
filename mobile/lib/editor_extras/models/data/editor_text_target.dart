// Kanso — editor_text_target.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

/// The open note's text, as the editor lets other features change it.
///
/// The editor (step 6) implements this for the note it shows and hands it
/// to the keys and actions in `EditorExtras`. Dictation and the inline
/// assistant write only through it, so every change is the editor's own:
/// one undo step, autosaved, and highlighted like typing.
abstract interface class EditorTextTarget {
  /// The note being edited.
  String get noteId;

  /// Monotonically increases after any text, selection, reload, or window
  /// change. Suggestions are bound to this revision and cannot outlive edits.
  int get revision;

  /// The first UTF-16 unit held by the editor's current window.
  int get windowOrigin;

  /// Whether this target still belongs to the live editor.
  bool get active;

  /// The editor's whole text now (the window being edited, for a long
  /// note), and its selection in that text.
  TextEditingValue get value;

  /// Notified when [value] changes.
  Listenable get changes;

  /// Replaces [range] (in [value]'s text) with [text] as one undo step, and
  /// puts the caret after it.
  void replace(TextRange range, String text);

  /// Replaces the selection with [text] as one undo step: inserts at the
  /// caret when nothing is selected.
  void insertAtSelection(String text);
}
