// Kanso — assist_editor_extras.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import '../../core/configs/kanso_icons.dart';
import '../../core/models/repos/abs/ai_repository.dart';
import '../../editor_extras/models/data/editor_extras.dart';
import '../models/data/assist_snapshot.dart';
import '../views/widgets/show_assist_sheet.dart';
import 'apply_suggestion.dart';

/// The inline assistant in the editor: an "Ask AI…" item in the selection
/// menu and a sparkles key above the keyboard. Both open the assistant
/// sheet; with nothing selected it offers Continue Writing and the note's
/// summary.
///
/// Nothing is offered while AI is off ([isEnabled]) or for a note the core
/// says is kept out of AI. Eligibility is asked once per note, in the
/// background: until it is known the note is taken to be eligible, and the
/// sheet says so if it is not.
class AssistEditorExtras implements EditorExtras {
  AssistEditorExtras({
    required this.ai,
    required this.isEnabled,
    required this.selectionLabel,
    required this.keyLabel,
  });

  final AiRepository ai;

  /// Whether AI is on now.
  final bool Function() isEnabled;

  /// What the selection menu item and the key say.
  final String selectionLabel;
  final String keyLabel;

  /// What the core said of each note asked about.
  final Map<String, bool> eligible = <String, bool>{};
  final Set<String> asking = <String>{};

  /// How much of the text before a caret a Continue Writing without a
  /// selection carries.
  static const int continueRoom = 6000;

  /// Whether the assistant is offered for the note [noteId].
  @visibleForTesting
  bool offered(String noteId) {
    if (!isEnabled()) {
      return false;
    }
    if (!eligible.containsKey(noteId) && asking.add(noteId)) {
      ai.isEligible(noteId).then((found) {
        asking.remove(noteId);
        found.fold((_) {}, (isEligible) => eligible[noteId] = isEligible);
      });
    }
    return eligible[noteId] ?? true;
  }

  @override
  List<EditorAccessoryExtra> accessoryKeys(EditorTextTarget target) {
    if (!offered(target.noteId)) {
      return const <EditorAccessoryExtra>[];
    }
    return <EditorAccessoryExtra>[
      EditorAccessoryExtra(
        id: 'assist',
        icon: KansoIcons.sparkles,
        semanticLabel: keyLabel,
        onTap: open,
      ),
    ];
  }

  @override
  List<EditorSelectionAction> selectionActions(EditorTextTarget target) {
    if (!offered(target.noteId) || target.value.selection.isCollapsed) {
      return const <EditorSelectionAction>[];
    }
    return <EditorSelectionAction>[
      EditorSelectionAction(
        id: 'assist',
        label: selectionLabel,
        onPressed: open,
      ),
    ];
  }

  /// Opens the assistant for the target's selection, or its caret.
  @visibleForTesting
  void open(BuildContext context, EditorTextTarget target) {
    final value = target.value;
    final text = value.text;
    final range = value.selection.isValid
        ? value.selection
        : TextSelection.collapsed(offset: text.length);
    final snapshot = AssistSnapshot(
      noteId: target.noteId,
      revision: target.revision,
      windowOrigin: target.windowOrigin,
      text: text,
      selection: TextRange(start: range.start, end: range.end),
    );
    // With no selection, Continue Writing continues the end of the text
    // before the caret.
    final room = snapshot.before.length > continueRoom
        ? snapshot.before.length - continueRoom
        : 0;
    final selection = range.isCollapsed
        ? snapshot.before.substring(room)
        : snapshot.selected;
    showAssistSheet(
      context,
      noteId: target.noteId,
      selection: selection,
      hasSelection: !range.isCollapsed,
      before: range.isCollapsed ? '' : snapshot.before,
      applyChanges: target.changes,
      canApply: () => assistSnapshotIsCurrent(target, snapshot),
      onApply: (placement, suggestion) =>
          applySuggestion(target, snapshot, placement, suggestion),
    );
  }

  @override
  void release(EditorTextTarget target) {}
}
