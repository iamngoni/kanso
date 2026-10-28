// Kanso — board_pane.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/browse_location.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/board_extensions.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/place_labels.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_empty_state.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../../core/views/widgets/kanso_notice.dart';
import '../../../core/views/widgets/kanso_segment.dart';
import '../../../core/views/widgets/kanso_segmented.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../gen/l10n/app_localizations.dart';
import '../../../journal/utils/open_made_note.dart';
import '../../models/data/board_drag.dart';
import '../../state/board/board_bloc.dart';
import 'board_column_view.dart';
import 'board_move_sheet.dart';

/// A board of the notes a list shows: a bar, the choice between the notes
/// by status and their tasks, the columns side by side (scrolling sideways
/// where they do not fit) and a hint under them.
///
/// On a phone a touch and hold on a card asks where to move it; where the
/// board is beside the list (a tablet, [draggable]) the card is dragged
/// to the column instead. It reads the `BoardBloc` and `LibraryIndexBloc`
/// above it.
class BoardPane extends StatelessWidget {
  const BoardPane({
    required this.place,
    required this.draggable,
    super.key,
  });

  final ListPlace place;

  /// Whether cards are dragged between columns (a tablet).
  final bool draggable;

  /// The widest a column is.
  static const double columnWidth = 300;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final bloc = context.read<BoardBloc>();
    return ColoredBox(
      color: colors.editor,
      child: BlocBuilder<LibraryIndexBloc, LibraryIndexState>(
        builder: (context, index) {
          return BlocBuilder<BoardBloc, BoardState>(
            builder: (context, state) {
              return Stack(
                children: <Widget>[
                  Column(
                    children: <Widget>[
                      KansoTopBar(
                        bordered: false,
                        leading: draggable
                            ? null
                            : KansoIconButton(
                                icon: KansoIcons.chevronLeft,
                                semanticLabel: l10n.boardClose,
                                onPressed: () => _close(context),
                              ),
                        title: l10n.boardTitle,
                        subtitle: placeTitle(l10n, place, index),
                        trailing: <Widget>[
                          if (draggable)
                            KansoIconButton(
                              icon: KansoIcons.close,
                              semanticLabel: l10n.boardClose,
                              onPressed: () => _close(context),
                            ),
                        ],
                      ),
                      Padding(
                        padding: const EdgeInsets.fromLTRB(
                          KansoSpacing.md,
                          KansoSpacing.xxs,
                          KansoSpacing.md,
                          KansoSpacing.xs,
                        ),
                        child: KansoSegmented<BoardKind>(
                          expanded: true,
                          semanticLabel: l10n.boardKindLabel,
                          selected: state.kind,
                          onChanged: (kind) =>
                              bloc.add(BoardEvent.kindChanged(kind)),
                          segments: <KansoSegment<BoardKind>>[
                            KansoSegment<BoardKind>(
                              value: BoardKind.notes,
                              label: l10n.boardNotes,
                            ),
                            KansoSegment<BoardKind>(
                              value: BoardKind.tasks,
                              label: l10n.boardTasks,
                            ),
                          ],
                        ),
                      ),
                      Expanded(child: _content(context, state)),
                      if (state.data != null && state.data!.cardCount > 0)
                        Padding(
                          padding: EdgeInsets.fromLTRB(
                            KansoSpacing.md,
                            KansoSpacing.xs,
                            KansoSpacing.md,
                            math.max(
                              KansoSpacing.sm,
                              MediaQuery.paddingOf(context).bottom,
                            ),
                          ),
                          child: Text(
                            _hint(l10n, state.kind),
                            textAlign: TextAlign.center,
                            style: context.typography.meta,
                          ),
                        ),
                    ],
                  ),
                  if (state.notice != null)
                    PositionedDirectional(
                      start: KansoSpacing.md,
                      end: KansoSpacing.md,
                      bottom:
                          MediaQuery.paddingOf(context).bottom +
                          KansoSpacing.xl,
                      child: KansoNotice(
                        message: switch (state.notice!) {
                          BoardNoticeChangedInNote() => l10n.boardChangedInNote,
                          BoardNoticeFailed() => l10n.boardMoveFailed,
                        },
                        dismissLabel: l10n.boardNoticeDismiss,
                        onDismiss: () =>
                            bloc.add(const BoardEvent.noticeDismissed()),
                      ),
                    ),
                ],
              );
            },
          );
        },
      ),
    );
  }

  String _hint(AppLocalizations l10n, BoardKind kind) {
    final notes = kind == BoardKind.notes;
    return draggable
        ? (notes ? l10n.boardHintNotesDrag : l10n.boardHintTasksDrag)
        : (notes ? l10n.boardHintNotes : l10n.boardHintTasks);
  }

  Widget _content(BuildContext context, BoardState state) {
    final l10n = context.l10n;
    final data = state.data;
    if (state.status == LoadStatus.failed && data == null) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(KansoSpacing.md),
          child: KansoMessageCard(
            icon: KansoIcons.fileText,
            title: l10n.boardFailedTitle,
            message: l10n.boardFailedMessage,
            actions: <Widget>[
              KansoButton(
                label: l10n.retry,
                onPressed: () =>
                    context.read<BoardBloc>().add(const BoardEvent.retried()),
              ),
            ],
          ),
        ),
      );
    }
    if (data == null) {
      return const SizedBox.shrink();
    }
    if (data.cardCount == 0) {
      return state.kind == BoardKind.notes
          ? KansoEmptyState(
              title: l10n.boardEmptyNotesTitle,
              message: l10n.boardEmptyNotesMessage,
            )
          : KansoEmptyState(
              title: l10n.boardEmptyTasksTitle,
              message: l10n.boardEmptyTasksMessage,
            );
    }
    return _Columns(
      data: data,
      draggable: draggable,
      onOpen: (card) => _open(context, card),
      onMove: (card) => _move(context, data, card),
      onDrop: (drag, lane) => context.read<BoardBloc>().add(
        BoardEvent.cardMoved(card: drag.card, lane: lane),
      ),
      onToggle: ({required card, required checked}) =>
          context.read<BoardBloc>().add(
            BoardEvent.cardMoved(
              card: card,
              lane: checked ? const BoardLane.done() : const BoardLane.todo(),
            ),
          ),
    );
  }

  void _open(BuildContext context, CardKey card) {
    // The editor opens a task's note at its line when it is there; the
    // note opens on a page of its own on a phone, so back is the board.
    openNoteById(context, card.noteIdOf, push: !draggable);
  }

  Future<void> _move(
    BuildContext context,
    BoardView data,
    CardKey card,
  ) async {
    final bloc = context.read<BoardBloc>();
    BoardLane? current;
    for (final BoardColumn column in data.columns) {
      if (column.cards.any((it) => it.key == card)) {
        current = column.lane;
      }
    }
    final lane = await BoardMoveSheet.show(
      context,
      columns: data.columns,
      current: current ?? data.columns.first.lane,
    );
    if (lane != null && lane != current) {
      bloc.add(BoardEvent.cardMoved(card: card, lane: lane));
    }
  }

  /// Back to the list the board is of.
  void _close(BuildContext context) {
    if (context.canPop()) {
      context.pop();
    } else {
      context.go(BrowseLocation.place(place).location);
    }
  }
}

class _Columns extends StatelessWidget {
  const _Columns({
    required this.data,
    required this.draggable,
    required this.onOpen,
    required this.onMove,
    required this.onDrop,
    required this.onToggle,
  });

  final BoardView data;
  final bool draggable;
  final void Function(CardKey card) onOpen;
  final void Function(CardKey card) onMove;
  final void Function(BoardDrag drag, BoardLane to) onDrop;
  final void Function({required CardKey card, required bool checked}) onToggle;

  @override
  Widget build(BuildContext context) {
    // On a phone the next column peeks in at the edge, so it shows that
    // the board scrolls sideways.
    final width = math.min(
      BoardPane.columnWidth,
      MediaQuery.sizeOf(context).width * 0.8,
    );
    return ListView.separated(
      scrollDirection: Axis.horizontal,
      padding: const EdgeInsets.symmetric(
        horizontal: KansoSpacing.md,
        vertical: KansoSpacing.xs,
      ),
      itemCount: data.columns.length,
      separatorBuilder: (_, _) => const SizedBox(width: KansoSpacing.sm),
      itemBuilder: (context, index) {
        return SizedBox(
          width: width,
          child: BoardColumnView(
            column: data.columns[index],
            draggable: draggable,
            onOpen: onOpen,
            onMove: onMove,
            onDrop: onDrop,
            onToggle: onToggle,
          ),
        );
      },
    );
  }
}
