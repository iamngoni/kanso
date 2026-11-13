// Kanso — when_assist.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';

import '../../../core/bridge/api/types.dart';
import '../../models/data/when_hint_view.dart';
import '../../models/repos/abs/when_repository.dart';
import 'kanso_markdown_controller.dart';

/// Typed dates in the note's field: the hint for a phrase at the end of the
/// task the caret is on, and the `@` list.
///
/// It listens to the [controller] and, after every change to the text or
/// the caret, asks the core what the caret's line holds. A hint is shown
/// only while the caret is at the end of the line, so that nothing is
/// offered to a phrase in the middle of an edit. Nothing is written until
/// [apply] (the hint tapped) or a row of the list is chosen; typing on, or
/// [dismissCompletion], leaves the text as it is.
///
/// The `@` list is the controller's own completion, so it shows in the same
/// popup, is stepped and chosen with the same keys, and is written by the
/// same command as a link's.
class WhenAssist extends ChangeNotifier {
  WhenAssist({
    required this.controller,
    required this.repository,
    required this.monthFirst,
    this.enabled,
  }) {
    controller.addListener(_sync);
    _sync();
  }

  final KansoMarkdownController controller;
  final WhenRepository repository;

  /// Whether `11/14` is the 14th of November.
  bool monthFirst;

  /// Whether the field is being typed in; nothing is offered when it
  /// is not (a note for reading). Always, when null.
  final bool Function()? enabled;

  WhenHintView? _hint;
  String? _dismissedBefore;
  bool _busy = false;
  bool _disposed = false;

  /// What the caret's line offers, or null.
  WhenHintView? get hint => _hint;

  /// Writes the hint's canonical token in place of the phrase. False when
  /// there is no hint.
  bool apply() {
    final view = _hint;
    if (view == null) {
      return false;
    }
    final origin = controller.windowStart + view.lineStart;
    _hint = null;
    final done = controller.run(
      EditCommand.insert(
        range: Range16(
          start: origin + view.hint.start,
          end: origin + view.hint.end,
        ),
        text: view.hint.canonical,
      ),
    );
    notifyListeners();
    return done;
  }

  /// Closes the `@` list until the text before the caret changes.
  void dismissCompletion() {
    final offered = controller.completion;
    if (offered == null || offered.kind != CompletionKind.date) {
      return;
    }
    _dismissedBefore = _beforeCaret();
    controller.offer(null);
  }

  /// Looks at the caret's line again: the field was focused or let go.
  void refresh() => _sync();

  String? _beforeCaret() {
    final selection = controller.value.selection;
    final text = controller.value.text;
    if (!selection.isValid || !selection.isCollapsed) {
      return null;
    }
    final caret = selection.extentOffset.clamp(0, text.length);
    final start = caret == 0 ? 0 : text.lastIndexOf('\n', caret - 1) + 1;
    return text.substring(start, caret);
  }

  void _sync() {
    if (_busy || _disposed || controller.isComposing) {
      return;
    }
    _busy = true;
    try {
      _update();
    } finally {
      _busy = false;
    }
  }

  void _update() {
    final text = controller.value.text;
    final selection = controller.value.selection;
    if (!(enabled?.call() ?? true) ||
        !selection.isValid ||
        !selection.isCollapsed) {
      _setHint(null);
      _withdraw();
      return;
    }
    final caret = selection.extentOffset.clamp(0, text.length);
    final lineStart = caret == 0 ? 0 : text.lastIndexOf('\n', caret - 1) + 1;
    final next = text.indexOf('\n', caret);
    final lineEnd = next < 0 ? text.length : next;
    final before = text.substring(lineStart, caret);

    // The `@` list, while nothing else is being completed.
    final offered = controller.completion;
    if (before != _dismissedBefore) {
      _dismissedBefore = null;
      final choices = offered == null || offered.kind == CompletionKind.date
          ? repository.choices(before)
          : null;
      if (choices != null) {
        final origin = controller.windowStart + lineStart;
        controller.offer(
          Completion(
            kind: CompletionKind.date,
            replace: Range16(
              start: origin + choices.start,
              end: controller.windowStart + caret,
            ),
            items: <CompletionItem>[
              for (final choice in choices.choices)
                CompletionItem(
                  label: choice.label,
                  detail: choice.detail,
                  insert: choice.insert,
                  newNote: false,
                ),
            ],
          ),
        );
      } else {
        _withdraw();
      }
    } else {
      _withdraw();
    }

    // The hint, at the end of the line only.
    if (text.substring(caret, lineEnd).trim().isNotEmpty) {
      _setHint(null);
      return;
    }
    final found = repository.hint(
      text.substring(lineStart, lineEnd),
      monthFirst: monthFirst,
    );
    _setHint(found == null ? null : (hint: found, lineStart: lineStart));
  }

  /// Takes the `@` list away, when it is what is offered.
  void _withdraw() {
    final offered = controller.completion;
    if (offered != null && offered.kind == CompletionKind.date) {
      controller.offer(null);
    }
  }

  void _setHint(WhenHintView? next) {
    if (next == _hint) {
      return;
    }
    _hint = next;
    notifyListeners();
  }

  @override
  void dispose() {
    _disposed = true;
    controller.removeListener(_sync);
    super.dispose();
  }
}
