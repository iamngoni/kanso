// Kanso — notebook_sharing_scope.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../injection.dart';
import '../../state/notebook_sharing/notebook_sharing_bloc.dart';

/// Gives [child] a [NotebookSharingBloc] for the notebook [notebookId]: who
/// last edited its notes and whether it is up to date, when it is shared.
/// A list that isn't a notebook's ([notebookId] null) gets one that holds
/// nothing. A new notebook gets a new bloc.
class NotebookSharingScope extends StatelessWidget {
  const NotebookSharingScope({
    required this.notebookId,
    required this.child,
    super.key,
  });

  final String? notebookId;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    final id = notebookId;
    return BlocProvider<NotebookSharingBloc>(
      key: ValueKey<String?>(id),
      create: (_) {
        final bloc = getIt<NotebookSharingBloc>();
        if (id != null) {
          bloc.add(NotebookSharingEvent.started(id));
        }
        return bloc;
      },
      child: child,
    );
  }
}
