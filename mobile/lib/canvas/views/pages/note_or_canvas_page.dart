// Kanso — note_or_canvas_page.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import '../../../injection.dart';
import '../../../note/views/pages/note_page.dart';
import '../../models/repos/abs/canvas_repository.dart';
import 'canvas_page.dart';

/// The page of a note's route: the canvas view for a canvas, the editor for
/// any other note. A canvas that is locked and closed still reads as a
/// canvas here; its page says so.
class NoteOrCanvasPage extends StatelessWidget {
  const NoteOrCanvasPage({
    required this.noteId,
    this.inPane = false,
    this.bodyLine,
    this.bodyLineEnd,
    this.cursor16,
    this.titleFirst = false,
    super.key,
  });

  final String noteId;
  final bool inPane;
  final int? bodyLine;
  final int? bodyLineEnd;
  final int? cursor16;
  final bool titleFirst;

  @override
  Widget build(BuildContext context) {
    if (getIt<CanvasRepository>().isCanvas(noteId)) {
      return CanvasPage(noteId: noteId, inPane: inPane);
    }
    return NotePage(
      noteId: noteId,
      inPane: inPane,
      bodyLine: bodyLine,
      bodyLineEnd: bodyLineEnd,
      cursor16: cursor16,
      titleFirst: titleFirst,
    );
  }
}
