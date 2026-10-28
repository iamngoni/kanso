// Kanso — fake_text_target.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:kanso/editor_extras/models/data/editor_text_target.dart';

/// A note's text as the editor lets features change it: each `replace` is
/// one undo step, counted in [undoSteps].
class FakeTextTarget implements EditorTextTarget {
  FakeTextTarget(String text, {int? caret})
    : notifier = ValueNotifier<TextEditingValue>(
        TextEditingValue(
          text: text,
          selection: TextSelection.collapsed(offset: caret ?? text.length),
        ),
      );

  final ValueNotifier<TextEditingValue> notifier;

  /// How many edits were made: each one step of the editor's undo.
  int undoSteps = 0;

  @override
  int revision = 0;

  @override
  int get windowOrigin => 0;

  @override
  bool active = true;

  @override
  String get noteId => 'note:fake';

  @override
  TextEditingValue get value => notifier.value;

  @override
  Listenable get changes => notifier;

  String get text => notifier.value.text;

  @override
  void replace(TextRange range, String text) {
    undoSteps++;
    revision++;
    notifier.value = TextEditingValue(
      text: value.text.replaceRange(range.start, range.end, text),
      selection: TextSelection.collapsed(offset: range.start + text.length),
    );
  }

  @override
  void insertAtSelection(String text) => replace(value.selection, text);
}
