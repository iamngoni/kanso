// Kanso — tasks_filter_sheet.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_check_row.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_sheet_group.dart';
import '../../state/tasks/tasks_bloc.dart';

/// Which notebook's tasks to show (All notebooks, or one of the notebooks)
/// and whether done ones show too, as the desktop's filter menu has them.
/// What is chosen applies at once; the sheet stays until it is done with.
class TasksFilterSheet extends StatelessWidget {
  const TasksFilterSheet({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final bloc = context.read<TasksBloc>();
    return BlocBuilder<LibraryIndexBloc, LibraryIndexState>(
      builder: (context, index) {
        return BlocBuilder<TasksBloc, TasksState>(
          builder: (context, state) {
            return ListView(
              shrinkWrap: true,
              padding: const EdgeInsets.only(bottom: KansoSpacing.sm),
              children: <Widget>[
                Padding(
                  padding: const EdgeInsetsDirectional.only(
                    start: KansoSpacing.md,
                    top: KansoSpacing.xs,
                  ),
                  child: Row(
                    children: <Widget>[
                      Expanded(
                        child: Text(
                          l10n.tasksFilterTitle,
                          style: context.typography.barTitle,
                        ),
                      ),
                      KansoPressable(
                        onTap: () => Navigator.of(context).maybePop(),
                        child: Padding(
                          padding: const EdgeInsets.all(KansoSpacing.sm),
                          child: Text(
                            l10n.noteListDone,
                            style: context.typography.body.copyWith(
                              color: colors.accent,
                            ),
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
                KansoSheetGroup(
                  title: l10n.tasksNotebooksHeader,
                  children: <Widget>[
                    KansoCheckRow(
                      label: l10n.tasksAllNotebooks,
                      checked: state.notebookId == null,
                      onTap: () =>
                          bloc.add(const TasksEvent.notebookChosen(null)),
                    ),
                    for (final notebook in index.notebooks)
                      KansoCheckRow(
                        label: notebook.role == NotebookRole.templates
                            ? l10n.libraryTemplates
                            : notebook.name,
                        checked: state.notebookId == notebook.id,
                        indent: notebook.depth * KansoSpacing.md,
                        onTap: () =>
                            bloc.add(TasksEvent.notebookChosen(notebook.id)),
                      ),
                  ],
                ),
                KansoSheetGroup(
                  children: <Widget>[
                    KansoCheckRow(
                      label: l10n.tasksShowCompleted,
                      checked: state.showDone,
                      onTap: () => bloc.add(
                        TasksEvent.showDoneChanged(showDone: !state.showDone),
                      ),
                    ),
                  ],
                ),
              ],
            );
          },
        );
      },
    );
  }
}
