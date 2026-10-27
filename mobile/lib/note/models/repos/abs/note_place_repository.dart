// Kanso — note_place_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../data/note_place.dart';

/// Where the person was in each note they edited, and which notes were just
/// made. Kept for this run of the app only, in memory: nothing about a note,
/// locked or not, reaches the disk through it.
abstract interface class NotePlaceRepository {
  /// Where [noteId] was left, if it was.
  NotePlace? placeOf(String noteId);

  /// Remembers that [noteId] was left at [place].
  void keep(String noteId, NotePlace place);

  /// Forgets [noteId]: its place and that it is new.
  void forget(String noteId);

  /// Remembers that [noteId] was just made, so that it opens ready to
  /// write in.
  void markOpenedNew(String noteId);

  /// Whether [noteId] was just made, once: asking again says no.
  bool takeOpenedNew(String noteId);
}
