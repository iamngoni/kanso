// Kanso — note_list_body.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/browse_location.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/place_labels.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../models/data/note_list_status.dart';
import '../../state/note_list/note_list_bloc.dart';
import 'empty_place_pane.dart';
import 'loading_list_pane.dart';
import 'note_list_view.dart';

/// What is under a list's bar: the shape of the list while it is read, why
/// it could not be, what an empty place will hold, or the notes.
class NoteListBody extends StatelessWidget {
  const NoteListBody({
    required this.state,
    required this.index,
    this.openNoteId,
    this.bottomClearance = 0,
    super.key,
  });

  final NoteListState state;
  final LibraryIndexState index;

  /// The note open beside the list.
  final String? openNoteId;

  /// How much room the notes leave below the last one for a button afloat.
  final double bottomClearance;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final place = state.place;
    return switch (state.status) {
      NoteListStatus.loading => const LoadingListPane(),
      NoteListStatus.failed => Center(
        child: Padding(
          padding: const EdgeInsets.all(KansoSpacing.md),
          child: KansoMessageCard(
            icon: KansoIcons.files,
            title: l10n.noteListFailedTitle,
            message: l10n.noteListFailedMessage,
            actions: <Widget>[
              KansoButton(
                label: l10n.retry,
                onPressed: () => context.read<NoteListBloc>().add(
                  const NoteListEvent.retried(),
                ),
              ),
            ],
          ),
        ),
      ),
      NoteListStatus.ready when state.rows.isEmpty => EmptyPlacePane(
        place: place,
        name: switch (place) {
          ListPlace_Notebook(:final id) =>
            index.notebookName(id) ?? placeTitle(l10n, place, index),
          ListPlace_Tag(:final id) =>
            index.tag(id)?.name ?? placeTitle(l10n, place, index),
          _ => placeTitle(l10n, place, index),
        },
        narrowed: state.options.narrowed,
      ),
      NoteListStatus.ready => NoteListView(
        state: state,
        index: index,
        location: BrowseLocation.place(place),
        openNoteId: openNoteId,
        bottomClearance: bottomClearance,
      ),
    };
  }
}
