// Kanso — voice_note_pane.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/utils/context_extensions.dart';
import '../../models/data/voice_note_status.dart';
import '../../state/dictation/dictation_bloc.dart';
import '../../state/voice_note/voice_note_bloc.dart';
import '../../utils/leave_voice_note.dart';
import 'voice_note_bar.dart';
import 'voice_note_body.dart';

/// The voice note: a new note being dictated, with what is heard as it is
/// heard, a large mic button, Cancel and Done.
///
/// Done stops the microphone, lets the recognizer finish what it heard, and
/// saves the words as a note, which then opens.
class VoiceNotePane extends StatelessWidget {
  const VoiceNotePane({super.key});

  @override
  Widget build(BuildContext context) {
    return MultiBlocListener(
      listeners: <BlocListener<dynamic, dynamic>>[
        // The recognizer is done, so the words are all there: keep them.
        BlocListener<DictationBloc, DictationState>(
          listenWhen: (before, now) => !before.finished && now.finished,
          listener: (context, state) => context.read<VoiceNoteBloc>().add(
            VoiceNoteEvent.saveRequested(state.text),
          ),
        ),
        BlocListener<VoiceNoteBloc, VoiceNoteState>(
          listenWhen: (before, now) =>
              before.status != now.status &&
              now.status == VoiceNoteStatus.saved,
          listener: (context, state) =>
              context.go(savedNoteAddress(state.noteId!)),
        ),
      ],
      child: Column(
        children: <Widget>[
          const VoiceNoteBar(),
          Expanded(
            child: ColoredBox(
              color: context.colors.editor,
              child: const VoiceNoteBody(),
            ),
          ),
        ],
      ),
    );
  }
}
