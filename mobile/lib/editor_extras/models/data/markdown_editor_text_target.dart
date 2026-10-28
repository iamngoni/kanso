// Kanso — markdown_editor_text_target.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

import '../../../core/bridge/api/types.dart';
import '../../../editor/views/controllers/kanso_markdown_controller.dart';
import 'editor_text_target.dart';

/// Adapts the windowed Markdown field to whole-note feature edits.
///
/// Ranges and selections exposed to features are always whole-note UTF-16
/// offsets. Writes go through the editor document's Rust command path so
/// they retain autosave, highlighting, and one-step undo behavior.
class MarkdownEditorTextTarget extends ChangeNotifier
    implements EditorTextTarget {
  MarkdownEditorTextTarget({required this.controller, required this.noteId}) {
    controller.addListener(changed);
  }

  final KansoMarkdownController controller;

  @override
  final String noteId;

  int _revision = 0;
  bool _active = true;
  @override
  int get revision => _revision;

  @override
  int get windowOrigin => controller.windowStart;

  @override
  bool get active => _active;

  @override
  TextEditingValue get value => TextEditingValue(
    text: controller.noteText,
    selection: controller.noteSelection,
  );

  @override
  Listenable get changes => this;

  void changed() {
    if (_active) {
      _revision++;
      notifyListeners();
    }
  }

  /// Marks a document reload or replacement even when the visible text is
  /// byte-for-byte equal to the previous snapshot.
  void invalidate() {
    if (_active) {
      _revision++;
      notifyListeners();
    }
  }

  @override
  void replace(TextRange range, String text) {
    if (!_active ||
        !range.isValid ||
        range.start < 0 ||
        range.end < range.start ||
        range.end > controller.noteLength) {
      return;
    }
    controller.run(
      EditCommand.insert(
        range: Range16(start: range.start, end: range.end),
        text: text,
      ),
    );
  }

  @override
  void insertAtSelection(String text) {
    final selection = controller.noteSelection;
    if (selection.isValid &&
        selection.start >= 0 &&
        selection.end <= controller.noteLength) {
      replace(
        TextRange(start: selection.start, end: selection.end),
        text,
      );
      return;
    }
    replace(TextRange.collapsed(controller.noteLength), text);
  }

  /// Stops future writes and makes any open suggestion stale.
  @override
  void dispose() {
    if (!_active) {
      return;
    }
    controller.removeListener(changed);
    _active = false;
    _revision++;
    notifyListeners();
    super.dispose();
  }
}
