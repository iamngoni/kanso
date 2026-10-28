// Kanso — voice_note_goldens_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/dictation/state/dictation/dictation_bloc.dart';

import '../../../helpers/golden.dart';
import '../../../helpers/kanso_viewport.dart';
import '../../helpers/voice_note_scene.dart';

void main() {
  final states = <String, DictationState>{
    'idle': VoiceNoteSeeds.idle,
    'listening': VoiceNoteSeeds.listening,
    'unavailable': VoiceNoteSeeds.unavailable,
    'denied': VoiceNoteSeeds.denied,
  };
  final viewports = <KansoViewport>[
    KansoViewport.phone,
    ...KansoViewport.goldenSet,
  ];
  for (final brightness in Brightness.values) {
    group('the voice note, ${brightness.name}', () {
      for (final viewport in viewports) {
        for (final MapEntry(key: label, value: seed) in states.entries) {
          testWidgets('$label at ${viewport.slug}', (tester) async {
            await expectGoldenOnBothPlatforms(
              tester,
              voiceNoteScene(seed),
              name: 'voice_note_${label}_${viewport.slug}_${brightness.name}',
              viewport: viewport,
              brightness: brightness,
            );
          });
        }
      }
    });
  }
}
