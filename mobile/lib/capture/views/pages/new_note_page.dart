// Kanso — new_note_page.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../note/models/data/new_note_start.dart';
import '../../../shell/state/note_creation/note_creation_bloc.dart';

/// `kanso://open/new`, from a widget or a shortcut: asks for a note as the
/// + button does, then gives way to it (the browse shell goes to each note
/// made). With [voice] the note starts in its text, to dictate into. If no
/// note can be made, the list says why.
class NewNotePage extends StatefulWidget {
  const NewNotePage({this.voice = false, super.key});

  final bool voice;

  @override
  State<NewNotePage> createState() => _NewNotePageState();
}

class _NewNotePageState extends State<NewNotePage> {
  @override
  void initState() {
    super.initState();
    context.read<NoteCreationBloc>().add(
      NoteCreationEvent.requested(
        const ListPlace.all(),
        start: widget.voice ? NewNoteStart.text : NewNoteStart.title,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return BlocListener<NoteCreationBloc, NoteCreationState>(
      listenWhen: (before, now) =>
          now.failure != null && before.failure != now.failure,
      listener: (context, state) => context.go(KansoRoutes.notes),
      child: ColoredBox(color: context.colors.editor),
    );
  }
}
