// Kanso — note_stage.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../../editor/views/widgets/note_editor_pane.dart';
import '../../models/data/note_mode.dart';
import '../../models/data/note_screen_status.dart';
import '../../state/note/note_bloc.dart';
import 'locked_note_pane.dart';
import 'note_body.dart';
import 'note_loading_pane.dart';
import 'note_not_found_pane.dart';

/// What is under a note's bar: the editor while it is edited, otherwise the
/// note read (title, facts and text), its lock, or why there is none to
/// show. It is the page of a note on a phone, the note pane of a tablet and
/// the body of a note's own window.
///
/// [shown] is the `NoteBloc`'s state, which the caller builds on; the bloc
/// is above this for the preview's own changes and for the editor.
class NoteStage extends StatelessWidget {
  const NoteStage({required this.shown, required this.gutter, super.key});

  final NoteState shown;

  /// The space either side of the text.
  final double gutter;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final note = shown.note;
    final readable = shown.status == NoteScreenStatus.ready && note != null;
    final editing = readable && shown.mode == NoteMode.edit;
    if (editing) {
      return NoteEditorPane(
        noteId: shown.noteId,
        gutter: gutter,
        caret: shown.caret,
        caretInBody: shown.caretInBody,
        scroll: shown.scroll,
        // A note made with no title is titled first, unless it is to be
        // dictated into.
        focusTitle:
            shown.openedNew && !shown.startInText && note.title.isEmpty,
      );
    }
    return BlocBuilder<LibraryIndexBloc, LibraryIndexState>(
      builder: (context, index) {
        return BlocBuilder<NoteBloc, NoteState>(
          builder: (context, state) {
            final note = state.note;
            return switch (state.status) {
              NoteScreenStatus.loading => NoteLoadingPane(gutter: gutter),
              NoteScreenStatus.ready when note != null => NoteBody(
                note: note,
                lines: state.lines,
                index: index,
                gutter: gutter,
                onEditAt: (offset) => context.read<NoteBloc>().add(
                  NoteEvent.modeChanged(NoteMode.edit, bodyCaret: offset),
                ),
              ),
              NoteScreenStatus.locked when note != null => LockedNotePane(
                noteId: note.id,
                title: note.title,
                gutter: gutter,
              ),
              NoteScreenStatus.ready ||
              NoteScreenStatus.locked ||
              NoteScreenStatus.notFound => const NoteNotFoundPane(),
              NoteScreenStatus.failed => Center(
                child: Padding(
                  padding: EdgeInsets.all(gutter),
                  child: KansoMessageCard(
                    icon: KansoIcons.fileText,
                    title: l10n.noteFailedTitle,
                    message: l10n.noteFailedMessage,
                    actions: <Widget>[
                      KansoButton(
                        label: l10n.retry,
                        onPressed: () => context.read<NoteBloc>().add(
                          const NoteEvent.retried(),
                        ),
                      ),
                    ],
                  ),
                ),
              ),
            };
          },
        );
      },
    );
  }
}
