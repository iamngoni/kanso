// Kanso — share_note_button.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../note/models/data/note_screen_status.dart';
import '../../../note/state/note/note_bloc.dart';
import 'share_note_sheet.dart';

/// The button in a note's top bar that opens its share sheet. It is there
/// once the note is read (a locked note too: its sheet says that it can't be
/// shared), and reads the `NoteBloc` above it.
class ShareNoteButton extends StatelessWidget {
  const ShareNoteButton({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<NoteBloc, NoteState>(
      buildWhen: (before, after) =>
          before.status != after.status || before.noteId != after.noteId,
      builder: (context, state) {
        final shareable =
            state.status == NoteScreenStatus.ready ||
            state.status == NoteScreenStatus.locked;
        if (!shareable) {
          return const SizedBox.shrink();
        }
        return KansoIconButton(
          icon: KansoIcons.share,
          semanticLabel: context.l10n.shareTitle,
          onPressed: () => ShareNoteSheet.show(context, state.noteId),
        );
      },
    );
  }
}
