// Kanso — show_assist_sheet.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/models/data/assist_action.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../injection.dart';
import '../../models/data/assist_placement.dart';
import '../../state/assist/assist_bloc.dart';
import 'assist_sheet.dart';

/// Opens the assistant over the screen for [selection] of the note [noteId]
/// ([before] is the text just before it, for context). With no text
/// selected, [selection] is the text a Continue Writing follows and
/// [hasSelection] is false. [onApply] puts an
/// accepted suggestion in the note; see [AssistSheet].
///
/// With [summarizeNote] it opens on the note's summary at once, and
/// [startWith] starts one selected action without the action list first.
/// The sheet's `AssistBloc` lives as long as the sheet.
Future<void> showAssistSheet(
  BuildContext context, {
  required String noteId,
  String selection = '',
  bool? hasSelection,
  String before = '',
  Listenable? applyChanges,
  bool Function()? canApply,
  bool Function(AssistPlacement placement, String suggestion)? onApply,
  bool summarizeNote = false,
  AssistAction? startWith,
}) {
  return showKansoBottomSheet<void>(
    context,
    builder: (context) => BlocProvider<AssistBloc>(
      create: (context) {
        final bloc = getIt<AssistBloc>();
        if (summarizeNote) {
          bloc.add(AssistEvent.summaryRequested(noteId));
        } else if (startWith case final action?) {
          bloc.add(
            AssistEvent.requested(
              noteId: noteId,
              action: action,
              before: before,
              selection: selection,
            ),
          );
        }
        return bloc;
      },
      child: AssistSheet(
        noteId: noteId,
        selection: selection,
        hasSelection: hasSelection ?? selection.isNotEmpty,
        before: before,
        applyChanges: applyChanges,
        canApply: canApply,
        onApply: onApply,
        summarizeNote: summarizeNote,
        startWith: startWith,
      ),
    ),
  );
}
