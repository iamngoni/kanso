// Kanso — open_cited_note.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:go_router/go_router.dart';

import '../../core/router/kanso_routes.dart';

/// The address of the note [id], by its id alone: the router takes it to the
/// note in All Notes.
String citedNoteLocation(String id) =>
    '${KansoRoutes.noteById}/${Uri.encodeComponent(id)}';

/// Opens the cited note [id].
///
/// On a phone, Ask stays under the note, so back returns to the answer. In a
/// sheet (a tablet) the sheet closes first, since the note opens in the
/// panes beneath it.
void openCitedNote(
  BuildContext context,
  String id, {
  required bool closeFirst,
}) {
  final router = GoRouter.of(context);
  if (closeFirst && router.canPop()) {
    router
      ..pop()
      ..go(citedNoteLocation(id));
    return;
  }
  router.push(citedNoteLocation(id));
}
