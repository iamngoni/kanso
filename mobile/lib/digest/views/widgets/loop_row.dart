// Kanso — loop_row.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/kanso_swipe_action.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_menu.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_swipe_actions.dart';

/// An open loop: its sentence, the note it is from and how long ago.
///
/// Swiping towards the end of the row closes it as done; the other way
/// shows "Not a loop" and "Make task". A long press offers the same in a
/// menu, and a screen reader gets them as actions. Tapping opens the note.
class LoopRow extends StatelessWidget {
  const LoopRow({
    required this.item,
    required this.onAction,
    required this.onOpen,
    super.key,
  });

  final OpenLoopItem item;
  final void Function(LoopAction action) onAction;
  final VoidCallback onOpen;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    final title = item.noteTitle.trim().isEmpty
        ? l10n.noteUntitled
        : item.noteTitle;
    final due = item.dueLabel;
    final meta = <String>[title, item.ageLabel, ?due].join(' · ');
    Future<void> menu(Offset at) async {
      final chosen = await showKansoMenu<Object>(
        context,
        at,
        <KansoMenuEntry<Object>>[
          KansoMenuEntry<Object>(
            LoopAction.done,
            l10n.digestLoopDone,
            icon: KansoIcons.check,
          ),
          KansoMenuEntry<Object>(
            LoopAction.makeTask,
            l10n.digestLoopMakeTask,
            icon: KansoIcons.listChecks,
          ),
          KansoMenuEntry<Object>(
            LoopAction.notALoop,
            l10n.digestLoopNotALoop,
            icon: KansoIcons.close,
          ),
          KansoMenuEntry<Object>(
            'open',
            l10n.digestLoopOpenNote,
            icon: KansoIcons.fileText,
            divider: true,
          ),
        ],
      );
      if (chosen is LoopAction) {
        onAction(chosen);
      } else if (chosen == 'open') {
        onOpen();
      }
    }

    final spoken = <String>[item.text, meta].join(', ');
    return Semantics(
      container: true,
      label: spoken,
      onTap: onOpen,
      customSemanticsActions: <CustomSemanticsAction, VoidCallback>{
        CustomSemanticsAction(label: l10n.digestLoopDone): () =>
            onAction(LoopAction.done),
        CustomSemanticsAction(label: l10n.digestLoopMakeTask): () =>
            onAction(LoopAction.makeTask),
        CustomSemanticsAction(label: l10n.digestLoopNotALoop): () =>
            onAction(LoopAction.notALoop),
        CustomSemanticsAction(label: l10n.digestLoopOpenNote): onOpen,
      },
      child: ExcludeSemantics(
        child: KansoSwipeActions(
          id: '${item.noteId}/${item.key}',
          startActions: <KansoSwipeAction>[
            KansoSwipeAction(
              icon: KansoIcons.check,
              label: l10n.digestLoopDone,
              background: colors.success,
              onPressed: () => onAction(LoopAction.done),
            ),
          ],
          endActions: <KansoSwipeAction>[
            KansoSwipeAction(
              icon: KansoIcons.listChecks,
              label: l10n.digestLoopMakeTask,
              background: colors.accent,
              onPressed: () => onAction(LoopAction.makeTask),
            ),
            KansoSwipeAction(
              icon: KansoIcons.close,
              label: l10n.digestLoopNotALoop,
              background: colors.warning,
              onPressed: () => onAction(LoopAction.notALoop),
            ),
          ],
          onCommitStart: () => onAction(LoopAction.done),
          child: GestureDetector(
            onLongPressStart: (details) => menu(details.globalPosition),
            child: ColoredBox(
              color: colors.list,
              child: KansoPressable(
                onTap: onOpen,
                child: Padding(
                  padding: const EdgeInsets.symmetric(
                    horizontal: KansoSpacing.md,
                    vertical: KansoSpacing.sm,
                  ),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: <Widget>[
                      Text(
                        item.text,
                        maxLines: 4,
                        overflow: TextOverflow.ellipsis,
                        style: typography.body.copyWith(color: colors.text),
                      ),
                      const SizedBox(height: KansoSpacing.xxs),
                      Text(
                        item.fromAi ? '$meta · ${l10n.digestLoopFromAi}' : meta,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: typography.meta.copyWith(
                          color: colors.textMuted,
                        ),
                      ),
                    ],
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}

/// The words for a kind of loop.
String loopKindLabel(BuildContext context, OpenLoopKind kind) {
  final l10n = context.l10n;
  return switch (kind) {
    OpenLoopKind.mine => l10n.digestKindMine,
    OpenLoopKind.waiting => l10n.digestKindWaiting,
    OpenLoopKind.question => l10n.digestKindQuestion,
  };
}
