// Kanso — board_card_tile.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_checkbox.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../tasks/views/widgets/task_due_chip.dart';

/// A card of a board. A note's card has its title and the start of its
/// text; a task's has a checkbox, its words, the note it is in and when it
/// is due.
///
/// A tap opens it; a touch and hold asks to move it ([onMove]) on a phone.
/// With [draggable] (a tablet) the touch and hold drags it instead, and
/// the card has a button for the move sheet. A screen reader gets the move
/// as an action of the card.
class BoardCardTile extends StatelessWidget {
  const BoardCardTile({
    required this.card,
    required this.onOpen,
    required this.onMove,
    this.onToggle,
    this.draggable = false,
    super.key,
  });

  final BoardCard card;
  final VoidCallback onOpen;
  final VoidCallback onMove;

  /// Ticks or unticks a task card; null for a note's.
  final ValueChanged<bool>? onToggle;
  final bool draggable;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final tile = Semantics(
      container: true,
      customSemanticsActions: <CustomSemanticsAction, VoidCallback>{
        CustomSemanticsAction(label: l10n.boardMoveTo): onMove,
      },
      child: KansoPressable(
        onTap: onOpen,
        onLongPress: draggable ? null : onMove,
        child: _Body(
          card: card,
          onToggle: onToggle,
          onMove: draggable ? onMove : null,
        ),
      ),
    );
    return tile;
  }
}

class _Body extends StatelessWidget {
  const _Body({required this.card, this.onToggle, this.onMove});

  final BoardCard card;
  final ValueChanged<bool>? onToggle;
  final VoidCallback? onMove;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    final isTask = card.key is CardKey_Task;
    final title = card.title.trim().isEmpty ? l10n.noteUntitled : card.title;
    final snippet = card.locked ? l10n.noteListLocked : card.snippet.trim();
    final due = card.dueChip;
    final move = onMove;
    return Container(
      padding: const EdgeInsetsDirectional.only(
        start: KansoSpacing.sm,
        top: KansoSpacing.sm,
        bottom: KansoSpacing.sm,
        end: KansoSpacing.xs,
      ),
      decoration: BoxDecoration(
        color: colors.elevated,
        borderRadius: KansoRadii.mediumBorder,
        border: Border.all(color: colors.divider),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: <Widget>[
          if (isTask) ...<Widget>[
            Padding(
              padding: const EdgeInsets.only(top: 2),
              child: Semantics(
                label: card.checked ? l10n.boardUntickTask : l10n.boardTickTask,
                child: KansoCheckbox(
                  value: card.checked,
                  onChanged: card.locked ? null : onToggle,
                ),
              ),
            ),
            const SizedBox(width: KansoSpacing.xs),
          ],
          Expanded(
            child: Padding(
              padding: const EdgeInsetsDirectional.only(end: KansoSpacing.xs),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: <Widget>[
                  Text(
                    title,
                    maxLines: 3,
                    overflow: TextOverflow.ellipsis,
                    style: typography.listTitle.copyWith(
                      color: card.checked ? colors.textMuted : colors.text,
                      decoration: card.checked
                          ? TextDecoration.lineThrough
                          : null,
                    ),
                  ),
                  if (isTask && card.noteTitle.trim().isNotEmpty)
                    Text(
                      card.noteTitle,
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: typography.meta,
                    ),
                  if (!isTask && snippet.isNotEmpty)
                    Padding(
                      padding: const EdgeInsets.only(top: 2),
                      child: Text(
                        snippet,
                        maxLines: 2,
                        overflow: TextOverflow.ellipsis,
                        style: typography.secondary,
                      ),
                    ),
                  if (due != null)
                    Padding(
                      padding: const EdgeInsets.only(top: 2),
                      child: Semantics(
                        label: l10n.boardTaskDue(due.label),
                        child: Tooltip(
                          message: due.tooltip,
                          child: ExcludeSemantics(
                            child: TaskDueChip(
                              label: due.label,
                              tone: due.tone,
                            ),
                          ),
                        ),
                      ),
                    ),
                ],
              ),
            ),
          ),
          if (move != null)
            KansoIconButton(
              icon: KansoIcons.ellipsis,
              semanticLabel: l10n.boardMoveTo,
              onPressed: move,
            ),
        ],
      ),
    );
  }
}
