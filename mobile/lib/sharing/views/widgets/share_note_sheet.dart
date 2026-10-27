// Kanso — share_note_sheet.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../injection.dart';
import '../../state/share_note/share_note_bloc.dart';
import 'share_note_view.dart';

/// The share sheet of the note [noteId]: who it is shared with and how,
/// whatever the size of the screen. It makes the `ShareNoteBloc` it shows
/// and gives it up when it closes.
class ShareNoteSheet extends StatelessWidget {
  const ShareNoteSheet({required this.noteId, super.key});

  final String noteId;

  /// Opens the share sheet of [noteId] over the app.
  static Future<void> show(BuildContext context, String noteId) {
    return showKansoBottomSheet<void>(
      context,
      builder: (context) => ShareNoteSheet(noteId: noteId),
    );
  }

  @override
  Widget build(BuildContext context) {
    return BlocProvider<ShareNoteBloc>(
      create: (context) =>
          getIt<ShareNoteBloc>()..add(ShareNoteEvent.started(noteId)),
      child: const ShareNoteView(),
    );
  }
}
