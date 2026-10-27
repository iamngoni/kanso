// Kanso — note_creation_listener.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/models/data/browse_location.dart';
import '../../../core/state/navigation_history/navigation_history_bloc.dart';
import '../../state/note_creation/note_creation_bloc.dart';

/// Opens each note the `NoteCreationBloc` makes, in the place it was made
/// from, ready to write in. It must be inside the app's router.
class NoteCreationListener extends StatelessWidget {
  const NoteCreationListener({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context) {
    return BlocListener<NoteCreationBloc, NoteCreationState>(
      listenWhen: (before, now) =>
          now.opening != null && before.opening != now.opening,
      listener: (context, state) {
        final opening = state.opening;
        if (opening == null) {
          return;
        }
        context.read<NoteCreationBloc>().add(
          const NoteCreationEvent.openingHandled(),
        );
        context.read<NavigationHistoryBloc>().add(
          NavigationHistoryEvent.visited(opening.noteId),
        );
        context.go(
          BrowseLocation.place(opening.place, noteId: opening.noteId).location,
        );
      },
      child: child,
    );
  }
}
