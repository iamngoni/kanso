// Kanso — note_liveness.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../models/repos/abs/library_repository.dart';

/// The id of the trash notebook: a note in it is gone from the person's
/// point of view.
const String trashNotebookId = 'trash';

/// Whether the note [id] is still there: it exists and is not in the trash.
Future<bool> isNoteLive(LibraryRepository library, String id) async {
  final note = await library.note(id);
  return note.fold(
    (_) => false,
    (found) => found.notebookId != trashNotebookId,
  );
}

/// Whether each of [candidates] is still there, asked of the library nearest
/// first, up to and including the first that is live and is not [current]:
/// the ones beyond cannot matter.
Future<Map<String, bool>> noteLiveness(
  LibraryRepository library,
  Iterable<String> candidates,
  String? current,
) async {
  final answers = <String, bool>{};
  for (final id in candidates) {
    if (id == current || answers.containsKey(id)) {
      continue;
    }
    final alive = await isNoteLive(library, id);
    answers[id] = alive;
    if (alive) {
      break;
    }
  }
  return answers;
}
