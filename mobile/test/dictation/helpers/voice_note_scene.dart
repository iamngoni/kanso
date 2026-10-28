// Kanso — voice_note_scene.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:kanso/dictation/models/data/dictation_status.dart';
import 'package:kanso/dictation/models/data/dictation_unavailable_reason.dart';
import 'package:kanso/dictation/state/dictation/dictation_bloc.dart';
import 'package:kanso/dictation/state/voice_note/voice_note_bloc.dart';
import 'package:kanso/dictation/views/widgets/voice_note_pane.dart';

import '../../helpers/close_bloc_later.dart';
import '../../helpers/mock_repositories.dart';
import 'mock_dictation_repository.dart';
import 'seeded_dictation_bloc.dart';

/// What the voice note's pictures show: a phone in each of its states.
abstract final class VoiceNoteSeeds {
  static const DictationState idle = DictationState();

  static const DictationState listening = DictationState(
    status: DictationStatus.listening,
    segments: <String>['Pick up the dry cleaning on Friday.'],
    finishedText: 'Pick up the dry cleaning on Friday.',
    text:
        'Pick up the dry cleaning on Friday. And call the plumber about the '
        'kitchen tap',
    partial: 'And call the plumber about the kitchen tap',
    level: 0.45,
  );

  static const DictationState unavailable = DictationState(
    status: DictationStatus.unavailable,
    reason: DictationUnavailableReason.notOnDevice,
  );

  static const DictationState denied = DictationState(
    status: DictationStatus.denied,
  );
}

/// The voice note over a `DictationBloc` in [seed].
Widget voiceNoteScene(DictationState seed) {
  final dictation = SeededDictationBloc(seed, SpokenRepository());
  final voice = VoiceNoteBloc(MockLibraryRepository());
  closeBlocLater(dictation);
  closeBlocLater(voice);
  return MultiBlocProvider(
    providers: <BlocProvider<dynamic>>[
      BlocProvider<DictationBloc>.value(value: dictation),
      BlocProvider<VoiceNoteBloc>.value(value: voice),
    ],
    child: const Scaffold(
      resizeToAvoidBottomInset: false,
      body: VoiceNotePane(),
    ),
  );
}
