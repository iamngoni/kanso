// Kanso — tasks_bar.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/models/data/browse_location.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../note_list/views/widgets/note_list_top_bar.dart';
import '../../models/data/tasks_status.dart';
import '../../state/tasks/tasks_bloc.dart';
import 'tasks_filter_sheet.dart';

/// The bar of the Tasks list: its title, how many tasks it holds, and the
/// filter.
class TasksBar extends StatelessWidget {
  const TasksBar({required this.state, super.key});

  final TasksState state;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final bloc = context.read<TasksBloc>();
    final index = context.read<LibraryIndexBloc>();
    return NoteListTopBar(
      title: l10n.libraryTasks,
      subtitle: state.status == TasksStatus.ready
          ? l10n.tasksCount(state.count)
          : null,
      onSearch: () => context.go(const BrowseLocation.search().location),
      optionsLabel: l10n.tasksFilter,
      onOptions: () => showKansoBottomSheet<void>(
        context,
        builder: (_) => MultiBlocProvider(
          providers: <BlocProvider<dynamic>>[
            BlocProvider<TasksBloc>.value(value: bloc),
            BlocProvider<LibraryIndexBloc>.value(value: index),
          ],
          child: const TasksFilterSheet(),
        ),
      ),
    );
  }
}
