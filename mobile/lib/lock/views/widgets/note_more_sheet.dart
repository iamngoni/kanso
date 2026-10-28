// Kanso — note_more_sheet.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart' hide LockState;

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_sheet_group.dart';
import '../../models/data/lock_target.dart';
import '../../models/data/note_menu_action.dart';
import '../../state/lock_status/lock_status_bloc.dart';
import '../../utils/note_menu_actions.dart';
import 'lock_item_sheet.dart';
import 'remove_lock_sheet.dart';

/// A note's More menu, as far as locks go: Lock note…, Remove lock… (the
/// note is locked and open) and Lock notes now (the notes are open). The
/// choice closes the menu and then runs.
class NoteMoreSheet extends StatelessWidget {
  const NoteMoreSheet({required this.actions, super.key});

  final List<NoteMenuAction> actions;

  /// Shows the menu of the note [noteId], whose lock is [lock], and runs
  /// what is chosen. Nothing is shown when there is nothing to offer.
  static Future<void> show(
    BuildContext context, {
    required String noteId,
    required LockState lock,
    required LockStatusBloc status,
  }) async {
    final actions = noteMenuActions(
      lock: lock,
      unlocked: status.state.unlocked,
    );
    if (actions.isEmpty) {
      return;
    }
    final chosen = await showKansoBottomSheet<NoteMenuAction>(
      context,
      builder: (_) => NoteMoreSheet(actions: actions),
    );
    if (chosen == null || !context.mounted) {
      return;
    }
    switch (chosen) {
      case NoteMenuAction.lockNote:
        await LockItemSheet.show(
          context,
          LockTarget.note(noteId),
          status: status,
        );
      case NoteMenuAction.removeLock:
        await RemoveLockSheet.show(
          context,
          LockTarget.note(noteId),
          status: status,
        );
      case NoteMenuAction.lockNow:
        status.add(const LockStatusEvent.lockedNow());
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        Padding(
          padding: const EdgeInsetsDirectional.fromSTEB(
            KansoSpacing.md,
            KansoSpacing.sm,
            KansoSpacing.md,
            KansoSpacing.xs,
          ),
          child: Text(l10n.lockMoreTitle, style: context.typography.barTitle),
        ),
        KansoSheetGroup(
          dividerIndent: KansoSettingsRow.iconInset,
          children: <Widget>[
            for (final action in actions)
              KansoSettingsRow(
                icon: switch (action) {
                  NoteMenuAction.lockNote => KansoIcons.lock,
                  NoteMenuAction.removeLock => KansoIcons.lockOpen,
                  NoteMenuAction.lockNow => KansoIcons.lock,
                },
                label: switch (action) {
                  NoteMenuAction.lockNote => l10n.lockNoteMenu,
                  NoteMenuAction.removeLock => l10n.lockRemoveNoteMenu,
                  NoteMenuAction.lockNow => l10n.lockNowMenu,
                },
                danger: action == NoteMenuAction.removeLock,
                onTap: () => Navigator.of(context).pop(action),
              ),
          ],
        ),
        const SizedBox(height: KansoSpacing.sm),
      ],
    );
  }
}
