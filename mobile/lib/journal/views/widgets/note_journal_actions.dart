// Kanso — note_journal_actions.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../note/state/note/note_bloc.dart';
import 'day_stepper.dart';

/// The journal's controls in the bar of a note that is in the Journal
/// notebook: the days before and after it, and the calendar. Nothing for
/// any other note. It needs a `JournalOpener` above it.
class NoteJournalActions extends StatelessWidget {
  const NoteJournalActions({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<LibraryIndexBloc, LibraryIndexState>(
      builder: (context, index) {
        return BlocBuilder<NoteBloc, NoteState>(
          buildWhen: (before, after) =>
              before.note?.notebookId != after.note?.notebookId ||
              before.noteId != after.noteId,
          builder: (context, state) {
            final notebook = index.notebook(state.note?.notebookId ?? '');
            if (notebook?.role != NotebookRole.journal) {
              return const SizedBox.shrink();
            }
            return DayStepper(openNote: state.noteId);
          },
        );
      },
    );
  }
}
