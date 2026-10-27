// Kanso — tasks_page.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/state/list_navigation/list_navigation_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../injection.dart';
import '../../state/tasks/tasks_bloc.dart';
import '../widgets/tasks_pane.dart';

/// The tasks of all the notes, read when the place is opened and given up
/// when it is left.
class TasksPage extends StatelessWidget {
  const TasksPage({this.openNoteId, super.key});

  /// The note open beside the list, in a window with room for both.
  final String? openNoteId;

  @override
  Widget build(BuildContext context) {
    return BlocProvider<TasksBloc>(
      create: (context) {
        // The keyboard steps through notes; this list has none.
        context.read<ListNavigationBloc>().add(
          const ListNavigationEvent.listShown(<String>[]),
        );
        return getIt<TasksBloc>()..add(const TasksEvent.started());
      },
      child: Scaffold(
        resizeToAvoidBottomInset: false,
        backgroundColor: context.colors.list,
        body: TasksPane(openNoteId: openNoteId),
      ),
    );
  }
}
