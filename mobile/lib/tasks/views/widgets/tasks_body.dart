// Kanso — tasks_body.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_empty_state.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../models/data/tasks_status.dart';
import '../../state/tasks/tasks_bloc.dart';
import 'tasks_list_view.dart';
import 'tasks_loading_pane.dart';

/// What is under the Tasks bar: the shape of the list while it is read, why
/// it could not be, why it is empty, or the tasks.
class TasksBody extends StatelessWidget {
  const TasksBody({required this.state, this.openNoteId, super.key});

  final TasksState state;

  /// The note open beside the list.
  final String? openNoteId;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return switch (state.status) {
      TasksStatus.loading => const TasksLoadingPane(),
      TasksStatus.failed => Center(
        child: Padding(
          padding: const EdgeInsets.all(KansoSpacing.md),
          child: KansoMessageCard(
            icon: KansoIcons.listChecks,
            title: l10n.tasksFailedTitle,
            message: l10n.tasksFailedMessage,
            actions: <Widget>[
              KansoButton(
                label: l10n.retry,
                onPressed: () =>
                    context.read<TasksBloc>().add(const TasksEvent.retried()),
              ),
            ],
          ),
        ),
      ),
      TasksStatus.ready when state.count == 0 => KansoEmptyState(
        title: switch (state.notebookId) {
          null => l10n.tasksEmptyTitle,
          final id => l10n.tasksEmptyNotebookTitle(
            context.read<LibraryIndexBloc>().state.notebookName(id) ??
                l10n.libraryNotebooks,
          ),
        },
        message: l10n.tasksEmptyMessage,
      ),
      TasksStatus.ready => TasksListView(state: state, openNoteId: openNoteId),
    };
  }
}
