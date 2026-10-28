// Kanso — note_list_lock_sync.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../lock/state/lock_status/lock_status_bloc.dart';
import '../../state/note_list/note_list_bloc.dart';

/// Has the `NoteListBloc` above it read the list again whenever the locked
/// notes open or close.
class NoteListLockSync extends StatelessWidget {
  const NoteListLockSync({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context) {
    return BlocListener<LockStatusBloc, LockStatusState>(
      listenWhen: (before, after) => before.unlocked != after.unlocked,
      listener: (context, state) => context.read<NoteListBloc>().add(
        const NoteListEvent.changesReceived(<String>[]),
      ),
      child: child,
    );
  }
}
