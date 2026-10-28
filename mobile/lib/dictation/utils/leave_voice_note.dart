// Kanso — leave_voice_note.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:go_router/go_router.dart';

import '../../core/router/kanso_routes.dart';

/// Leaves the voice note for the page it was opened from, or for the notes
/// when it was the first page (a link opened it).
void leaveVoiceNote(BuildContext context) {
  if (context.canPop()) {
    context.pop();
  } else {
    context.go(KansoRoutes.notes);
  }
}

/// The address of the note [id], as the router opens it.
String savedNoteAddress(String id) =>
    '${KansoRoutes.noteById}/${Uri.encodeComponent(id)}';
