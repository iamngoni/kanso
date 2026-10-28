// Kanso — notebook_actions_sheet.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_sheet_group.dart';
import '../../../lock/state/lock_status/lock_status_bloc.dart';
import 'notebook_lock_actions.dart';

/// One action of a notebook's menu.
class NotebookSheetAction {
  const NotebookSheetAction({
    required this.label,
    required this.icon,
    required this.onTap,
    this.destructive = false,
  });

  final String label;
  final KansoIcons icon;
  final VoidCallback onTap;

  /// Whether it is drawn in the danger colour.
  final bool destructive;
}

/// A notebook's menu, opened by a long press in the library: its name and
/// its actions. A choice closes the menu and then runs.
class NotebookActionsSheet extends StatelessWidget {
  const NotebookActionsSheet({required this.notebook, super.key});

  final NotebookRow notebook;

  /// Shows the menu of [notebook] over the whole app.
  static Future<void> show(BuildContext context, NotebookRow notebook) {
    final index = context.read<LibraryIndexBloc>();
    final status = context.read<LockStatusBloc>();
    return showKansoBottomSheet<void>(
      context,
      builder: (_) => MultiBlocProvider(
        providers: <BlocProvider<dynamic>>[
          BlocProvider<LibraryIndexBloc>.value(value: index),
          BlocProvider<LockStatusBloc>.value(value: status),
        ],
        child: NotebookActionsSheet(notebook: notebook),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final actions = notebookLockActions(context, notebook);
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
          child: Text(
            notebook.name,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: context.typography.barTitle,
          ),
        ),
        KansoSheetGroup(
          dividerIndent: KansoSettingsRow.iconInset,
          children: <Widget>[
            for (final action in actions)
              KansoSettingsRow(
                icon: action.icon,
                label: action.label,
                danger: action.destructive,
                onTap: () {
                  Navigator.of(context).pop();
                  action.onTap();
                },
              ),
          ],
        ),
        const SizedBox(height: KansoSpacing.sm),
      ],
    );
  }
}
