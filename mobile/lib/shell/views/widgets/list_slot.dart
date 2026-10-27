// Kanso — list_slot.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import '../../../core/models/data/browse_location.dart';
import '../../../note_list/views/pages/note_list_page.dart';
import '../../../search/views/pages/search_page.dart';

/// The list pane of a wide window: the notes of the place, or the results
/// of the search, that [location] says.
class ListSlot extends StatelessWidget {
  const ListSlot({required this.location, super.key});

  final BrowseLocation? location;

  @override
  Widget build(BuildContext context) {
    return switch (location) {
      PlaceLocation(:final place, :final noteId) => NoteListPage(
        place: place,
        openNoteId: noteId,
        beside: true,
      ),
      SearchLocation(
        :final query,
        :final scope,
        :final notebookId,
        :final noteId,
      ) =>
        SearchPage(
          query: query,
          scope: scope,
          notebookId: notebookId,
          openNoteId: noteId,
        ),
      null => const SizedBox.shrink(),
    };
  }
}
