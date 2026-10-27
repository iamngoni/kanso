// Kanso — note_pane.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/browse_location.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/state/navigation_history/navigation_history_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../../editor/views/widgets/note_editor_pane.dart';
import '../../models/data/note_mode.dart';
import '../../models/data/note_screen_status.dart';
import '../../state/note/note_bloc.dart';
import 'locked_note_pane.dart';
import 'note_body.dart';
import 'note_bottom_bar.dart';
import 'note_info_sheet.dart';
import 'note_loading_pane.dart';
import 'note_mode_shortcuts.dart';
import 'note_mode_toggle.dart';
import 'note_not_found_pane.dart';
import 'note_top_bar.dart';

/// A note: a bar, its title, facts and text, or why there is none to show.
/// It reads the `NoteBloc` above it.
///
/// On a phone ([inPane] false) the note has a page of its own, with a way
/// back to the list and a bar of back, forward, search and settings under
/// it. On a tablet it is a pane beside the list, and the bar above it has
/// back and forward.
class NotePane extends StatelessWidget {
  const NotePane({required this.inPane, super.key});

  final bool inPane;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final gutter = inPane ? KansoSpacing.lg : KansoSpacing.md;
    final history = context.read<NavigationHistoryBloc>();
    return NoteModeShortcuts(
      child: ColoredBox(
        color: context.colors.editor,
        child: BlocBuilder<NavigationHistoryBloc, NavigationHistoryState>(
          builder: (context, steps) {
            void goBack() =>
                history.add(const NavigationHistoryEvent.backRequested());
            void goForward() =>
                history.add(const NavigationHistoryEvent.forwardRequested());
            return BlocBuilder<NoteBloc, NoteState>(
              buildWhen: (before, now) =>
                  before.mode != now.mode ||
                  before.status != now.status ||
                  before.note != now.note,
              builder: (context, shown) {
                final note = shown.note;
                final readable =
                    shown.status == NoteScreenStatus.ready && note != null;
                final editing = readable && shown.mode == NoteMode.edit;
                // The keyboard has the foot of the screen: the bar of the
                // page gives way to it and the editor's own bar.
                final keyboardUp = MediaQuery.viewInsetsOf(context).bottom > 0;
                return Column(
                  children: <Widget>[
                    NoteTopBar(
                      phone: !inPane,
                      onBack: () => leave(context),
                      onGoBack: steps.canGoBack ? goBack : null,
                      onGoForward: steps.canGoForward ? goForward : null,
                      trailing: <Widget>[
                        if (readable) ...<Widget>[
                          NoteModeToggle(
                            mode: shown.mode,
                            onChanged: (mode) => context.read<NoteBloc>().add(
                              NoteEvent.modeChanged(mode),
                            ),
                          ),
                          KansoIconButton(
                            icon: KansoIcons.info,
                            semanticLabel: l10n.noteInfo,
                            onPressed: () => showKansoBottomSheet<void>(
                              context,
                              builder: (_) => NoteInfoSheet(
                                note: note,
                                index: context.read<LibraryIndexBloc>().state,
                              ),
                            ),
                          ),
                        ],
                      ],
                    ),
                    Expanded(
                      child: editing
                          ? NoteEditorPane(
                              noteId: shown.noteId,
                              gutter: gutter,
                              caret: shown.caret,
                              caretInBody: shown.caretInBody,
                              scroll: shown.scroll,
                              // A note made with no title is titled first.
                              focusTitle: shown.openedNew && note.title.isEmpty,
                            )
                          : BlocBuilder<LibraryIndexBloc, LibraryIndexState>(
                              builder: (context, index) {
                                return BlocBuilder<NoteBloc, NoteState>(
                                  builder: (context, state) {
                                    final note = state.note;
                                    return switch (state.status) {
                                      NoteScreenStatus.loading =>
                                        NoteLoadingPane(
                                          gutter: gutter,
                                        ),
                                      NoteScreenStatus.ready
                                          when note != null =>
                                        NoteBody(
                                          note: note,
                                          lines: state.lines,
                                          index: index,
                                          gutter: gutter,
                                          onEditAt: (offset) =>
                                              context.read<NoteBloc>().add(
                                                NoteEvent.modeChanged(
                                                  NoteMode.edit,
                                                  bodyCaret: offset,
                                                ),
                                              ),
                                        ),
                                      NoteScreenStatus.locked
                                          when note != null =>
                                        LockedNotePane(
                                          title: note.title,
                                          gutter: gutter,
                                        ),
                                      NoteScreenStatus.ready ||
                                      NoteScreenStatus.locked ||
                                      NoteScreenStatus.notFound =>
                                        const NoteNotFoundPane(),
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
                                                onPressed: () => context
                                                    .read<NoteBloc>()
                                                    .add(
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
                            ),
                    ),
                    if (!inPane && !(editing && keyboardUp))
                      NoteBottomBar(
                        onGoBack: steps.canGoBack ? goBack : null,
                        onGoForward: steps.canGoForward ? goForward : null,
                        onSearch: () =>
                            context.go(const BrowseLocation.search().location),
                        onSettings: () => context.push(KansoRoutes.settings),
                      ),
                  ],
                );
              },
            );
          },
        ),
      ),
    );
  }

  /// Back to the list the note was opened from.
  void leave(BuildContext context) {
    if (context.canPop()) {
      context.pop();
    } else {
      context.go(KansoRoutes.notes);
    }
  }
}
