// Kanso — tasks_pane.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../state/tasks/tasks_bloc.dart';
import 'tasks_bar.dart';
import 'tasks_body.dart';
import 'tasks_notice_line.dart';

/// The Tasks place: a bar, and under it the tasks in their groups, or while
/// they are read the shape of them, or why there are none. [openNoteId] is
/// the note open beside the list.
class TasksPane extends StatelessWidget {
  const TasksPane({this.openNoteId, super.key});

  final String? openNoteId;

  @override
  Widget build(BuildContext context) {
    return ColoredBox(
      color: context.colors.list,
      child: BlocBuilder<TasksBloc, TasksState>(
        builder: (context, state) {
          final notice = state.notice;
          return Stack(
            children: <Widget>[
              Column(
                children: <Widget>[
                  TasksBar(state: state),
                  Expanded(
                    child: TasksBody(state: state, openNoteId: openNoteId),
                  ),
                ],
              ),
              if (notice != null)
                PositionedDirectional(
                  start: KansoSpacing.md,
                  end: KansoSpacing.md,
                  bottom:
                      MediaQuery.paddingOf(context).bottom + KansoSpacing.md,
                  child: TasksNoticeLine(
                    notice: notice,
                    onDismiss: () => context.read<TasksBloc>().add(
                      const TasksEvent.noticeDismissed(),
                    ),
                  ),
                ),
            ],
          );
        },
      ),
    );
  }
}
