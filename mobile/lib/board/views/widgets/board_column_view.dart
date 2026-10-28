// Kanso — board_column_view.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../models/data/board_drag.dart';
import '../../utils/board_labels.dart';
import 'board_card_tile.dart';

/// One column of a board: its name and how many cards it has, then the
/// cards, scrolling on their own. With [draggable] cards can be dropped on
/// it.
class BoardColumnView extends StatelessWidget {
  const BoardColumnView({
    required this.column,
    required this.draggable,
    required this.onOpen,
    required this.onMove,
    required this.onDrop,
    required this.onToggle,
    super.key,
  });

  final BoardColumn column;
  final bool draggable;
  final void Function(CardKey card) onOpen;
  final void Function(CardKey card) onMove;
  final void Function(BoardDrag drag, BoardLane to) onDrop;
  final void Function({required CardKey card, required bool checked}) onToggle;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final name = l10n.laneName(column);
    return DragTarget<BoardDrag>(
      onWillAcceptWithDetails: (details) => details.data.from != column.lane,
      onAcceptWithDetails: (details) => onDrop(details.data, column.lane),
      builder: (context, candidates, rejected) {
        final hovered = candidates.isNotEmpty;
        return Semantics(
          container: true,
          label: l10n.boardColumnLabel(name, column.cards.length),
          child: Container(
            decoration: BoxDecoration(
              color: colors.list,
              borderRadius: KansoRadii.largeBorder,
              border: Border.all(
                color: hovered ? colors.accent : colors.divider,
              ),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: <Widget>[
                Padding(
                  padding: const EdgeInsets.fromLTRB(
                    KansoSpacing.sm,
                    KansoSpacing.sm,
                    KansoSpacing.sm,
                    KansoSpacing.xs,
                  ),
                  child: ExcludeSemantics(
                    child: Row(
                      children: <Widget>[
                        Expanded(
                          child: Text(
                            name,
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                            style: context.typography.sectionLabel,
                          ),
                        ),
                        Text(
                          '${column.cards.length}',
                          style: context.typography.meta,
                        ),
                      ],
                    ),
                  ),
                ),
                Expanded(
                  child: ListView.separated(
                    padding: const EdgeInsets.fromLTRB(
                      KansoSpacing.xs,
                      0,
                      KansoSpacing.xs,
                      KansoSpacing.xs,
                    ),
                    itemCount: column.cards.length,
                    separatorBuilder: (_, _) =>
                        const SizedBox(height: KansoSpacing.xs),
                    itemBuilder: (context, index) {
                      final card = column.cards[index];
                      final tile = BoardCardTile(
                        card: card,
                        draggable: draggable,
                        onOpen: () => onOpen(card.key),
                        onMove: () => onMove(card.key),
                        onToggle: card.key is CardKey_Task
                            ? (checked) =>
                                  onToggle(card: card.key, checked: checked)
                            : null,
                      );
                      if (!draggable) {
                        return tile;
                      }
                      return LongPressDraggable<BoardDrag>(
                        data: BoardDrag(card: card.key, from: column.lane),
                        feedback: Material(
                          type: MaterialType.transparency,
                          child: SizedBox(
                            width: 280,
                            child: Opacity(opacity: 0.9, child: tile),
                          ),
                        ),
                        childWhenDragging: Opacity(opacity: 0.4, child: tile),
                        child: tile,
                      );
                    },
                  ),
                ),
              ],
            ),
          ),
        );
      },
    );
  }
}
