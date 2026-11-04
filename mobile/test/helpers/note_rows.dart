// Kanso — note_rows.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:kanso/core/bridge/api/types.dart';

/// A note row for tests: the facts a list shows.
NoteRow noteRow(String id, {String? title, bool locked = false}) {
  return NoteRow(
    canvas: false,
    id: id,
    rev: '1-$id',
    title: title ?? 'Note $id',
    snippet: locked ? '' : 'The text of $id.',
    notebookId: 'b_1',
    status: 'active',
    tags: const <String>[],
    pinned: false,
    createdMs: 1700000000000,
    updatedMs: 1700000000000,
    tasksDone: 0,
    tasksTotal: 0,
    locked: locked,
  );
}
