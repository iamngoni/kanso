// Kanso — open_assist_action.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../core/models/data/assist_action.dart';
import '../../core/utils/context_extensions.dart';
import '../../editor_extras/models/data/editor_text_target.dart';
import '../models/data/assist_snapshot.dart';
import '../views/widgets/show_assist_sheet.dart';
import 'apply_suggestion.dart';

/// How much of the text before a caret a Continue Writing without a
/// selection carries when the sheet is opened by the editor key.
const int _continueRoom = 6000;

/// Opens the assistant for [target], or starts [action] directly when one is
/// given. A direct action follows the desktop's rules: rewrites need text
/// selected, while Continue Writing uses the paragraph before the caret.
Future<void> openAssistAction(
  BuildContext context,
  EditorTextTarget target, {
  AssistAction? action,
}) async {
  if (!target.active) {
    return;
  }
  final value = target.value;
  final text = value.text;
  final selection = value.selection.isValid
      ? value.selection
      : TextSelection.collapsed(offset: text.length);

  if (action == null) {
    final snapshot = AssistSnapshot(
      noteId: target.noteId,
      revision: target.revision,
      windowOrigin: target.windowOrigin,
      text: text,
      selection: TextRange(start: selection.start, end: selection.end),
    );
    final room = snapshot.before.length > _continueRoom
        ? snapshot.before.length - _continueRoom
        : 0;
    final offered = selection.isCollapsed
        ? snapshot.before.substring(room)
        : snapshot.selected;
    await showAssistSheet(
      context,
      noteId: target.noteId,
      selection: offered,
      hasSelection: !selection.isCollapsed,
      before: selection.isCollapsed ? '' : snapshot.before,
      applyChanges: target.changes,
      canApply: () => assistSnapshotIsCurrent(target, snapshot),
      onApply: (placement, suggestion) =>
          applySuggestion(target, snapshot, placement, suggestion),
    );
    return;
  }

  var range = TextRange(start: selection.start, end: selection.end);
  if (range.isCollapsed) {
    if (action != AssistAction.continueWriting) {
      _notice(context, context.l10n.assistSelectText);
      return;
    }
    range = paragraphBefore(text, range.end);
    if (range.isCollapsed) {
      _notice(context, context.l10n.assistContinueAfterText);
      return;
    }
  }
  if (range.textInside(text).trim().isEmpty) {
    _notice(context, context.l10n.assistSelectText);
    return;
  }

  final snapshot = AssistSnapshot(
    noteId: target.noteId,
    revision: target.revision,
    windowOrigin: target.windowOrigin,
    text: text,
    selection: TextRange(start: range.start, end: range.end),
  );
  await showAssistSheet(
    context,
    noteId: target.noteId,
    selection: snapshot.selected,
    hasSelection: !selection.isCollapsed,
    before: snapshot.before,
    startWith: action,
    applyChanges: target.changes,
    canApply: () => assistSnapshotIsCurrent(target, snapshot),
    onApply: (placement, suggestion) =>
        applySuggestion(target, snapshot, placement, suggestion),
  );
}

/// The paragraph that ends at [caret]: from the blank line before it (or the
/// start of the text) to the caret. Collapsed when that is only blank space.
@visibleForTesting
TextRange paragraphBefore(String text, int caret) {
  final end = caret.clamp(0, text.length);
  final blank = text.substring(0, end).lastIndexOf('\n\n');
  var start = blank < 0 ? 0 : blank + 2;
  final raw = text.substring(start, end);
  start += raw.length - raw.trimLeft().length;
  if (text.substring(start, end).trim().isEmpty) {
    return TextRange.collapsed(end);
  }
  return TextRange(start: start, end: end);
}

void _notice(BuildContext context, String message) {
  ScaffoldMessenger.maybeOf(context)?.showSnackBar(
    SnackBar(content: Text(message)),
  );
}
