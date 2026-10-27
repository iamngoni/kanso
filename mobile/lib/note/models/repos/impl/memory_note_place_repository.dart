// Kanso — memory_note_place_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:collection';

import 'package:injectable/injectable.dart';

import '../../data/note_place.dart';
import '../abs/note_place_repository.dart';

/// [NotePlaceRepository] in memory: the 64 notes edited last.
@LazySingleton(as: NotePlaceRepository)
class MemoryNotePlaceRepository implements NotePlaceRepository {
  /// How many notes it remembers.
  static const int capacity = 64;

  final LinkedHashMap<String, NotePlace> places =
      LinkedHashMap<String, NotePlace>();
  final Set<String> newNotes = <String>{};

  @override
  NotePlace? placeOf(String noteId) {
    final place = places.remove(noteId);
    if (place != null) {
      // Looked at counts as used.
      places[noteId] = place;
    }
    return place;
  }

  @override
  void keep(String noteId, NotePlace place) {
    places
      ..remove(noteId)
      ..[noteId] = place;
    while (places.length > capacity) {
      places.remove(places.keys.first);
    }
  }

  @override
  void forget(String noteId) {
    places.remove(noteId);
    newNotes.remove(noteId);
  }

  @override
  void markOpenedNew(String noteId) {
    newNotes.add(noteId);
  }

  @override
  bool takeOpenedNew(String noteId) => newNotes.remove(noteId);
}
