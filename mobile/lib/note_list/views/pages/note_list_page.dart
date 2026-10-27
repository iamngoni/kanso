// Kanso — note_list_page.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/state/list_navigation/list_navigation_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../injection.dart';
import '../../state/note_list/note_list_bloc.dart';
import '../widgets/note_list_navigation_sync.dart';
import '../widgets/note_list_pane.dart';

/// The notes of a library place. Each place has its own list, read when it
/// is opened and given up when it is left.
class NoteListPage extends StatelessWidget {
  const NoteListPage({
    required this.place,
    this.openNoteId,
    this.beside = false,
    super.key,
  });

  final ListPlace place;

  /// The note open beside the list, in a window with room for both.
  final String? openNoteId;

  /// Whether the list is beside the note it opens, in a window with room
  /// for both, rather than a page of its own.
  final bool beside;

  @override
  Widget build(BuildContext context) {
    return BlocProvider<NoteListBloc>(
      // A new place is a new list: nothing of the last one is kept.
      key: ValueKey<ListPlace>(place),
      create: (context) {
        // Nothing of the last list is left for the keyboard to step to.
        context.read<ListNavigationBloc>().add(
          const ListNavigationEvent.listShown(<String>[]),
        );
        return getIt<NoteListBloc>()..add(NoteListEvent.started(place));
      },
      child: NoteListNavigationSync(
        child: Scaffold(
          resizeToAvoidBottomInset: false,
          backgroundColor: context.colors.list,
          body: NoteListPane(openNoteId: openNoteId, beside: beside),
        ),
      ),
    );
  }
}
