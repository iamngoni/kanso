// Kanso — notebook_gate_host.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../lock/views/widgets/notebook_unlock_gate.dart';

/// Puts the unlock gate around [child] when [place] is a locked notebook,
/// and nothing around it otherwise.
class NotebookGateHost extends StatelessWidget {
  const NotebookGateHost({required this.place, required this.child, super.key});

  final ListPlace place;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    final here = place;
    if (here is! ListPlace_Notebook) {
      return child;
    }
    final notebook = context.select<LibraryIndexBloc, NotebookRow?>(
      (index) => index.state.notebooks
          .where((book) => book.id == here.id && book.locked)
          .firstOrNull,
    );
    return notebook == null
        ? child
        : NotebookUnlockGate(
            key: ValueKey<String>(notebook.id),
            notebook: notebook,
            child: child,
          );
  }
}
