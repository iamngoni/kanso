// Kanso — note_list_pane.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../capture/models/data/capture_first_row.dart';
import '../../../capture/views/widgets/capture_sheet.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/state/capture_inbox/capture_inbox_bloc.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_large_title.dart';
import '../../../shell/models/data/note_creation_failure.dart';
import '../../../shell/state/note_creation/note_creation_bloc.dart';
import '../../state/note_list/note_list_bloc.dart';
import 'new_note_button.dart';
import 'note_list_bar.dart';
import 'note_list_body.dart';
import 'note_list_notice.dart';

/// The notes of a place: a bar, and under it the list grouped by date, or
/// while it is read the shape of one, or why there is none.
///
/// It reads the `NoteListBloc`, `LibraryIndexBloc` and `NoteCreationBloc`
/// above it. On a phone the bar has the menu that opens the library, search
/// and more, and a button for a new note is afloat in the corner: the list
/// has room below its last row for it, and its rows fade out beneath it, so
/// nothing is hidden behind it for good. On a tablet with the library
/// beside the list, the bar has more and a button for a new note. [beside]
/// says the list is beside the note, and [openNoteId] is the note open
/// there.
class NoteListPane extends StatelessWidget {
  const NoteListPane({this.openNoteId, this.beside = false, super.key});

  final String? openNoteId;
  final bool beside;

  /// How far below its last row a phone's list scrolls: the floating
  /// button, with a gutter above and below it.
  static const double floatingClearance =
      KansoSizes.floatingButton + 2 * KansoSpacing.md;

  /// How far above the clearance the fade begins.
  static const double fadeReach = 56;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final bottom = MediaQuery.paddingOf(context).bottom;
    return ColoredBox(
      color: colors.list,
      child: BlocBuilder<LibraryIndexBloc, LibraryIndexState>(
        builder: (context, index) {
          return BlocBuilder<CaptureInboxBloc, CaptureInboxState>(
            buildWhen: (before, after) => before.problems != after.problems,
            builder: (context, inbox) {
              return BlocBuilder<NoteCreationBloc, NoteCreationState>(
                builder: (context, creation) {
                  return BlocBuilder<NoteListBloc, NoteListState>(
                    builder: (context, state) {
                      final bloc = context.read<NoteListBloc>();
                      final error = state.actionError;
                      void newNote() => context.read<NoteCreationBloc>().add(
                        NoteCreationEvent.requested(
                          state.place,
                          pinned: state.options.pinnedOnly,
                        ),
                      );
                      // Holding the new-note button offers the camera, photos
                      // and PDFs, each saved as a note of its own.
                      void capture() => CaptureSheet.show(
                        context,
                        first: CaptureFirstRow(
                          icon: KansoIcons.plus,
                          label: l10n.noteListNewNote,
                          onTap: newNote,
                        ),
                        onSource: (source) =>
                            context.read<NoteCreationBloc>().add(
                              NoteCreationEvent.captured(source),
                            ),
                      );
                      // The button is afloat over a phone's list, except while
                      // notes are being chosen.
                      final floating = !beside && !state.selecting;
                      final message = switch (creation.failure) {
                        NoteCreationFailure.locked =>
                          l10n.noteListNewNoteLocked,
                        NoteCreationFailure.failed =>
                          l10n.noteListNewNoteFailed,
                        null when creation.problems.isNotEmpty =>
                          creation.problems.join('\n'),
                        // What other apps shared and was left out of a note.
                        null when inbox.problems.isNotEmpty =>
                          inbox.problems.join('\n'),
                        null => switch (error) {
                          null => null,
                          LockedError() => l10n.noteListChangeLocked,
                          _ => l10n.noteListChangeFailed,
                        },
                      };
                      // The notice says one thing at a time, and dismissing
                      // it clears that thing.
                      void dismissNotice() {
                        if (creation.failure != null ||
                            creation.problems.isNotEmpty) {
                          context.read<NoteCreationBloc>().add(
                            const NoteCreationEvent.noticeDismissed(),
                          );
                        } else if (inbox.problems.isNotEmpty) {
                          context.read<CaptureInboxBloc>().add(
                            const CaptureInboxEvent.problemsDismissed(),
                          );
                        } else {
                          bloc.add(const NoteListEvent.errorDismissed());
                        }
                      }

                      return PopScope(
                        // Back, on a phone that has one, ends choosing
                        // before it leaves.
                        canPop: !state.selecting,
                        onPopInvokedWithResult: (didPop, result) {
                          if (!didPop) {
                            bloc.add(const NoteListEvent.selectionCleared());
                          }
                        },
                        child: Stack(
                          children: <Widget>[
                            KansoLargeTitleScroll(
                              builder: (context, collapsed) => Column(
                                children: <Widget>[
                                  NoteListBar(
                                    state: state,
                                    index: index,
                                    newNote: beside ? newNote : null,
                                    capture: beside ? capture : null,
                                    // A phone's list has its title large, above
                                    // the notes, until it scrolls away.
                                    titleShown:
                                        floating &&
                                            NoteListBody.showsLargeTitle(state)
                                        ? collapsed
                                        : null,
                                  ),
                                  Expanded(
                                    child: NoteListBody(
                                      largeTitle: floating,
                                      state: state,
                                      index: index,
                                      openNoteId: openNoteId,
                                      bottomClearance: floating
                                          ? floatingClearance - KansoSpacing.md
                                          : 0,
                                    ),
                                  ),
                                ],
                              ),
                            ),
                            if (floating) ...<Widget>[
                              PositionedDirectional(
                                start: 0,
                                end: 0,
                                bottom: 0,
                                height: bottom + floatingClearance + fadeReach,
                                child: IgnorePointer(
                                  child: DecoratedBox(
                                    decoration: BoxDecoration(
                                      gradient: LinearGradient(
                                        begin: Alignment.topCenter,
                                        end: Alignment.bottomCenter,
                                        colors: <Color>[
                                          colors.list.withValues(alpha: 0),
                                          colors.list.withValues(alpha: 0.9),
                                          colors.list.withValues(alpha: 0.96),
                                        ],
                                        stops: const <double>[0, 0.65, 1],
                                      ),
                                    ),
                                  ),
                                ),
                              ),
                              PositionedDirectional(
                                start: 0,
                                end: 0,
                                bottom: bottom + KansoSpacing.md,
                                child: Center(
                                  child: NewNoteButton(
                                    onPressed: newNote,
                                    onLongPress: capture,
                                  ),
                                ),
                              ),
                            ],
                            if (message != null)
                              PositionedDirectional(
                                start: KansoSpacing.md,
                                end: KansoSpacing.md,
                                // Over the new-note button, when there is one.
                                bottom:
                                    bottom +
                                    KansoSpacing.md +
                                    (floating
                                        ? KansoSizes.floatingButton +
                                              KansoSpacing.sm
                                        : 0),
                                child: NoteListNotice(
                                  message: message,
                                  onDismiss: dismissNotice,
                                ),
                              ),
                          ],
                        ),
                      );
                    },
                  );
                },
              );
            },
          );
        },
      ),
    );
  }
}
