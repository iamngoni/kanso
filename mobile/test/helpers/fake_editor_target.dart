// Kanso — fake_editor_target.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:kanso/editor_extras/models/data/editor_text_target.dart';

/// An editor's text for tests of what writes into it.
class FakeEditorTarget extends ChangeNotifier implements EditorTextTarget {
  FakeEditorTarget(String text, {TextSelection? selection, this.noteId = 'n1'})
    : value = TextEditingValue(
        text: text,
        selection: selection ?? TextSelection.collapsed(offset: text.length),
      );

  @override
  final String noteId;

  @override
  int revision = 0;

  @override
  int get windowOrigin => 0;

  @override
  bool active = true;

  @override
  TextEditingValue value;

  /// How many edits were made, each one undo step.
  int edits = 0;

  @override
  Listenable get changes => this;

  @override
  void replace(TextRange range, String text) {
    edits++;
    revision++;
    final next = value.text.replaceRange(range.start, range.end, text);
    value = TextEditingValue(
      text: next,
      selection: TextSelection.collapsed(offset: range.start + text.length),
    );
    notifyListeners();
  }

  @override
  void insertAtSelection(String text) => replace(
    TextRange(start: value.selection.start, end: value.selection.end),
    text,
  );

  /// The person types: not an edit of the assistant’s.
  void type(String text) {
    revision++;
    value = TextEditingValue(
      text: text,
      selection: TextSelection.collapsed(offset: text.length),
    );
    notifyListeners();
  }
}
