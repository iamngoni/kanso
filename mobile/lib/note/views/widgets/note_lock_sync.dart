// Kanso — note_lock_sync.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../lock/state/lock_status/lock_status_bloc.dart';
import '../../state/note/note_bloc.dart';

/// Has the `NoteBloc` above it read the note again whenever the locked
/// notes open or close, so a locked note shows its text once they are open
/// and its card once they are closed.
class NoteLockSync extends StatelessWidget {
  const NoteLockSync({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context) {
    return BlocListener<LockStatusBloc, LockStatusState>(
      listenWhen: (before, after) => before.unlocked != after.unlocked,
      listener: (context, state) {
        final note = context.read<NoteBloc>();
        note.add(NoteEvent.changesReceived(<String>[note.state.noteId]));
      },
      child: child,
    );
  }
}
