// Kanso — apply_suggestion.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/painting.dart';

import '../../editor_extras/models/data/editor_text_target.dart';
import '../models/data/assist_placement.dart';
import '../models/data/assist_snapshot.dart';

/// Puts the accepted [suggestion] into the note [target] edits, as one undo
/// step, and gives whether it did.
///
/// Nothing is changed if the editor changed, reloaded, moved its window, or
/// switched notes since [snapshot] was taken.
bool applySuggestion(
  EditorTextTarget target,
  AssistSnapshot snapshot,
  AssistPlacement placement,
  String suggestion,
) {
  if (!assistSnapshotIsCurrent(target, snapshot)) {
    return false;
  }
  final now = target.value.text;
  final range = snapshot.selection;
  switch (placement) {
    case AssistPlacement.replace:
      target.replace(range, suggestion);
    case AssistPlacement.insertBelow:
      final upTo = now.substring(0, range.end);
      final gap = upTo.isEmpty || upTo.endsWith('\n\n')
          ? ''
          : upTo.endsWith('\n')
          ? '\n'
          : '\n\n';
      target.replace(TextRange.collapsed(range.end), '$gap$suggestion');
  }
  return true;
}

/// Whether [target] is still exactly the live editor the snapshot names.
bool assistSnapshotIsCurrent(
  EditorTextTarget target,
  AssistSnapshot snapshot,
) {
  if (!target.active ||
      target.noteId != snapshot.noteId ||
      target.revision != snapshot.revision ||
      target.windowOrigin != snapshot.windowOrigin ||
      snapshot.selection.start < 0 ||
      snapshot.selection.end < snapshot.selection.start) {
    return false;
  }
  final now = target.value.text;
  final range = snapshot.selection;
  return range.end <= now.length &&
      range.textBefore(now) == snapshot.before &&
      range.textInside(now) == snapshot.selected;
}
