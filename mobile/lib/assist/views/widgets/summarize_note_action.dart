// Kanso — summarize_note_action.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../ai_settings/state/ai_status/ai_status_bloc.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../editor_extras/models/data/editor_text_target.dart';
import '../../models/data/assist_placement.dart';
import '../../models/data/assist_snapshot.dart';
import '../../utils/apply_suggestion.dart';
import 'show_assist_sheet.dart';

/// "Summarize this note", a row for the note's info menu. It is drawn only
/// while AI is on, and opens the note's summary in a sheet to copy, and,
/// when [target] (the editor) is given, to insert at the caret.
class SummarizeNoteAction extends StatelessWidget {
  const SummarizeNoteAction({required this.noteId, this.target, super.key});

  final String noteId;

  /// The note's editor, where the summary can be inserted; none while the
  /// note is only being read.
  final EditorTextTarget? target;

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<AiStatusBloc, AiStatusState>(
      buildWhen: (before, now) => before.enabled != now.enabled,
      builder: (context, state) {
        if (!state.enabled) {
          return const SizedBox.shrink();
        }
        return KansoSettingsRow(
          icon: KansoIcons.sparkles,
          label: context.l10n.assistSummarizeNote,
          onTap: () => summarizeNote(context, noteId, target: target),
        );
      },
    );
  }
}

/// Opens the summary of the note [noteId] in a sheet; with the editor
/// [target] it can be inserted at the caret.
Future<void> summarizeNote(
  BuildContext context,
  String noteId, {
  EditorTextTarget? target,
}) {
  final editor = target;
  final snapshot = editor == null
      ? null
      : AssistSnapshot(
          noteId: editor.noteId,
          revision: editor.revision,
          windowOrigin: editor.windowOrigin,
          text: editor.value.text,
          selection: editor.value.selection.isValid
              ? TextRange.collapsed(editor.value.selection.end)
              : TextRange.collapsed(editor.value.text.length),
        );
  return showAssistSheet(
    context,
    noteId: noteId,
    summarizeNote: true,
    onApply: editor == null || snapshot == null || !editor.active
        ? null
        : (placement, text) => applySuggestion(
            editor,
            snapshot,
            AssistPlacement.insertBelow,
            text,
          ),
    applyChanges: editor?.changes,
    canApply: editor == null || snapshot == null
        ? null
        : () => assistSnapshotIsCurrent(editor, snapshot),
  );
}
