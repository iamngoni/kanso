// Kanso — note_list_bar.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/browse_location.dart';
import '../../../core/models/data/search_scope_kind.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/place_labels.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_selection_bar.dart';
import '../../models/data/note_list_status.dart';
import '../../state/note_list/note_list_bloc.dart';
import 'list_options_sheet.dart';
import 'move_sheet.dart';
import 'note_actions_sheet.dart';
import 'note_list_top_bar.dart';

/// The bar of a list: its title and what to do with the list, or, while
/// several notes are chosen, how many and what to do with them.
class NoteListBar extends StatelessWidget {
  const NoteListBar({
    required this.state,
    required this.index,
    this.newNote,
    this.capture,
    this.titleShown,
    super.key,
  });

  final NoteListState state;
  final LibraryIndexState index;

  /// Makes a new note, from a button in the bar; null where the list has a
  /// button afloat over it instead.
  final VoidCallback? newNote;

  /// Offers the camera, photos and PDFs, from holding that button.
  final VoidCallback? capture;

  /// When the title shows, for a list whose large title scrolls with it.
  final ValueListenable<bool>? titleShown;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final bloc = context.read<NoteListBloc>();
    if (state.selecting) {
      final chosen = state.selected.toList();
      return KansoSelectionBar(
        count: chosen.length,
        pinned: state.selectionPinned,
        onDone: () => bloc.add(const NoteListEvent.selectionCleared()),
        onPin: state.inTrash
            ? null
            : () => bloc.add(
                NoteListEvent.changeRequested(
                  const NoteChange.togglePin(),
                  chosen,
                ),
              ),
        onMove: () => MoveSheet.show(context, bloc: bloc, ids: chosen),
        onTrash: state.inTrash
            ? null
            : () => bloc.add(
                NoteListEvent.changeRequested(const NoteChange.trash(), chosen),
              ),
        onMore: () =>
            NoteActionsSheet.show(context, bloc: bloc, ids: chosen),
      );
    }
    final place = state.place;
    return NoteListTopBar(
      onNewNote: newNote,
      onNewNoteHeld: capture,
      titleShown: titleShown,
      title: placeTitle(l10n, place, index),
      subtitle: state.status == NoteListStatus.ready
          ? l10n.noteListCount(state.rows.length)
          : null,
      // Search from a notebook looks in that notebook.
      onSearch: () => context.go(
        BrowseLocation.search(
          scope: place is ListPlace_Notebook
              ? SearchScopeKind.notebook
              : SearchScopeKind.all,
          notebookId: place is ListPlace_Notebook ? place.id : null,
        ).location,
      ),
      onOptions: () => showKansoBottomSheet<void>(
        context,
        builder: (_) => BlocProvider<NoteListBloc>.value(
          value: bloc,
          child: const ListOptionsSheet(),
        ),
      ),
    );
  }
}
