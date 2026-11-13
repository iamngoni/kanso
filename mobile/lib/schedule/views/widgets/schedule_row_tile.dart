// Kanso — schedule_row_tile.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/schedule_types.dart';
import '../../../core/bridge/api/types.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../state/schedule/schedule_bloc.dart';
import 'schedule_actions_sheet.dart';
import 'schedule_item_tile.dart';

/// A [ScheduleItemTile] wired to the Schedule: the box ticks through the
/// bloc, the row opens the note (at the task's line), and the actions sheet
/// ticks, opens or moves to a day the person picks.
class ScheduleRowTile extends StatelessWidget {
  const ScheduleRowTile({required this.row, super.key});

  final ScheduleRow row;

  Future<void> _more(BuildContext context) async {
    final bloc = context.read<ScheduleBloc>();
    final action = await ScheduleActionsSheet.show(context, row);
    if (action == null || !context.mounted) {
      return;
    }
    switch (action) {
      case ScheduleAction.tick:
        bloc.add(ScheduleEvent.ticked(row));
      case ScheduleAction.open:
        _open(context);
      case ScheduleAction.move:
        final day = await ScheduleDaySheet.show(
          context,
          row,
          weekStart: bloc.state.weekStart,
        );
        if (day != null) {
          bloc.add(ScheduleEvent.moved(row, day));
        }
    }
  }

  void _open(BuildContext context) {
    final line = row.line;
    final path = '${KansoRoutes.notes}/${KansoRoutes.note}/${row.noteId}';
    unawaited(
      context.push(
        line == null ? path : '$path?${KansoRoutes.taskLineParameter}=$line',
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final notebooks = context.select<LibraryIndexBloc, List<NotebookRow>>(
      (bloc) => bloc.state.notebooks,
    );
    var notebook = '';
    for (final candidate in notebooks) {
      if (candidate.id == row.bookId) {
        notebook = candidate.name;
      }
    }
    return ScheduleItemTile(
      row: row,
      notebook: notebook,
      onTick: () => context.read<ScheduleBloc>().add(ScheduleEvent.ticked(row)),
      onOpen: () => _open(context),
      onMore: () => unawaited(_more(context)),
    );
  }
}
