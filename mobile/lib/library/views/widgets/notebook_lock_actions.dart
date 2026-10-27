// Kanso — notebook_lock_actions.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../lock/models/data/lock_target.dart';
import '../../../lock/state/lock_status/lock_status_bloc.dart';
import '../../../lock/utils/notebook_lock_rule.dart';
import '../../../lock/views/widgets/lock_item_sheet.dart';
import '../../../lock/views/widgets/remove_lock_sheet.dart';
import '../../../lock/views/widgets/unlock_sheet.dart';
import 'notebook_actions_sheet.dart';

/// The lock items of a notebook's menu, as the desktop has them: Remove
/// Notebook Lock… when the notebook has a lock of its own (after unlocking
/// it, when the notes are closed), Lock Notebook… when it isn't locked, and
/// nothing when it is locked through a notebook it is inside.
///
/// Reads the `LibraryIndexBloc` and `LockStatusBloc` above [context]. The
/// actions do not close the menu; whatever shows them does that before
/// calling `onTap`.
List<NotebookSheetAction> notebookLockActions(
  BuildContext context,
  NotebookRow notebook,
) {
  final l10n = context.l10n;
  final status = context.read<LockStatusBloc>();
  final all = context.read<LibraryIndexBloc>().state.notebooks;
  // The sheets go over the whole app, from a place that outlives the menu
  // this is called for.
  final host = Navigator.of(context, rootNavigator: true).overlay!.context;
  final target = LockTarget.notebook(notebook.id, notebook.name);
  switch (notebookLockRule(notebook, all)) {
    case NotebookLockRule.lock:
      return <NotebookSheetAction>[
        NotebookSheetAction(
          label: l10n.lockNotebookMenu,
          icon: KansoIcons.lock,
          onTap: () => LockItemSheet.show(host, target, status: status),
        ),
      ];
    case NotebookLockRule.remove:
      return <NotebookSheetAction>[
        if (!status.state.unlocked)
          NotebookSheetAction(
            label: l10n.lockUnlockNotebookMenu,
            icon: KansoIcons.lockOpen,
            onTap: () => UnlockSheet.open(host, target: target, status: status),
          ),
        NotebookSheetAction(
          label: l10n.lockRemoveNotebookMenu,
          icon: KansoIcons.lockOpen,
          destructive: true,
          onTap: () async {
            // Taking a lock off needs the notebook open.
            if (!status.state.unlocked &&
                !await UnlockSheet.open(host, target: target, status: status)) {
              return;
            }
            if (host.mounted) {
              await RemoveLockSheet.show(host, target, status: status);
            }
          },
        ),
      ];
    case NotebookLockRule.none:
      return const <NotebookSheetAction>[];
  }
}
