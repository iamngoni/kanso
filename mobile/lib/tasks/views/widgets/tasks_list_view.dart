// Kanso — tasks_list_view.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/browse_location.dart';
import '../../../core/state/navigation_history/navigation_history_bloc.dart';
import '../../../core/state/sync_status/sync_status_bloc.dart';
import '../../../core/state/undo_notice/undo_notice_bloc.dart';
import '../../../core/views/widgets/kanso_divider.dart';
import '../../../core/views/widgets/kanso_refresh_control.dart';
import '../../../core/views/widgets/kanso_section_header.dart';
import '../../state/tasks/tasks_bloc.dart';
import 'task_row.dart';
import 'tasks_keyboard.dart';

/// The tasks under their headers, scrolling; pulling the list down syncs.
///
/// Pressing a task's box ticks it, pressing the rest opens its note.
/// [openNoteId] is the note open beside the list.
class TasksListView extends StatelessWidget {
  const TasksListView({required this.state, this.openNoteId, super.key});

  final TasksState state;
  final String? openNoteId;

  @override
  Widget build(BuildContext context) {
    final bloc = context.read<TasksBloc>();
    // Only an account has anything to pull: a phone that is signed out
    // just bounces.
    final signedIn = context.select<SyncStatusBloc, bool>(
      (sync) => sync.state.signedIn,
    );
    void open(TaskItem task) {
      const here = BrowseLocation.tasks();
      context.read<NavigationHistoryBloc>().add(
        NavigationHistoryEvent.visited(task.noteId),
      );
      context.go(here.withNote(task.noteId).location);
    }

    return TasksKeyboard(
      tasks: state.items,
      onTick: (task) => bloc.add(TasksEvent.ticked(task)),
      onOpen: open,
      onUndo: () => context.read<UndoNoticeBloc>().add(
        const UndoNoticeEvent.undoPressed(),
      ),
      builder: (context, focusedKey) {
        return CustomScrollView(
          physics: signedIn ? const AlwaysScrollableScrollPhysics() : null,
          slivers: <Widget>[
            if (signedIn)
              KansoRefreshControl(onRefresh: () => syncNow(context)),
            for (final section in state.board.groups)
              SliverMainAxisGroup(
                slivers: <Widget>[
                  SliverToBoxAdapter(
                    child: KansoSectionHeader(label: section.label),
                  ),
                  SliverList.builder(
                    itemCount: section.items.length,
                    itemBuilder: (context, at) {
                      final task = section.items[at];
                      return Column(
                        key: ValueKey<String>(task.key),
                        children: <Widget>[
                          if (at > 0)
                            const KansoDivider(indent: KansoSpacing.md),
                          TaskRow(
                            task: task,
                            open: task.noteId == openNoteId,
                            focused: task.key == focusedKey,
                            onTick: () => bloc.add(TasksEvent.ticked(task)),
                            onOpen: () => open(task),
                          ),
                        ],
                      );
                    },
                  ),
                ],
              ),
            SliverToBoxAdapter(
              child: SizedBox(
                height: MediaQuery.paddingOf(context).bottom + KansoSpacing.md,
              ),
            ),
          ],
        );
      },
    );
  }

  /// Asks for a sync, and waits for it to end.
  Future<void> syncNow(BuildContext context) {
    final sync = context.read<SyncStatusBloc>()
      ..add(const SyncStatusEvent.syncRequested());
    return sync.stream.firstWhere((state) => !state.syncing);
  }
}
