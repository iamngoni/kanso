// Kanso — voice_note_page.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../injection.dart';
import '../../state/dictation/dictation_bloc.dart';
import '../../state/voice_note/voice_note_bloc.dart';
import '../widgets/voice_note_pane.dart';

/// A new note, dictated, at `/voice-note`: the phone listens through its own
/// recognizer, and Done saves the words as a note.
class VoiceNotePage extends StatefulWidget {
  const VoiceNotePage({super.key});

  @override
  State<VoiceNotePage> createState() => _VoiceNotePageState();
}

class _VoiceNotePageState extends State<VoiceNotePage> {
  bool checked = false;
  DictationBloc? dictation;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    if (!checked) {
      checked = true;
      dictation = context.read<DictationBloc>();
      dictation!.add(const DictationEvent.checked());
    }
  }

  @override
  void dispose() {
    if (dictation?.state.active ?? false) {
      dictation!.add(const DictationEvent.cancelled());
    }
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return BlocProvider<VoiceNoteBloc>(
      create: (context) => getIt<VoiceNoteBloc>(),
      child: Scaffold(
        backgroundColor: context.colors.editor,
        body: const VoiceNotePane(),
      ),
    );
  }
}
