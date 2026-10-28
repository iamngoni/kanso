// Kanso — voice_settings_goldens_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:kanso/dictation/models/data/dictation_capability.dart';
import 'package:kanso/dictation/models/data/dictation_permission.dart';
import 'package:kanso/dictation/models/data/dictation_unavailable_reason.dart';
import 'package:kanso/voice_settings/models/data/voice_settings.dart';
import 'package:kanso/voice_settings/state/voice_settings/voice_settings_bloc.dart';
import 'package:kanso/voice_settings/views/widgets/voice_settings_pane.dart';

import '../../../dictation/helpers/mock_dictation_repository.dart';
import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/golden.dart';
import '../../../helpers/kanso_viewport.dart';
import '../../helpers/mock_voice_settings_repository.dart';

class SeededVoiceSettingsBloc extends VoiceSettingsBloc {
  SeededVoiceSettingsBloc(VoiceSettingsState seed)
    : super(MockVoiceSettingsRepository(), SpokenRepository().repository) {
    emit(seed);
  }
}

Widget _scene(VoiceSettingsState seed) {
  final bloc = SeededVoiceSettingsBloc(seed);
  closeBlocLater(bloc);
  return BlocProvider<VoiceSettingsBloc>.value(
    value: bloc,
    child: const Scaffold(
      resizeToAvoidBottomInset: false,
      body: VoiceSettingsPane(),
    ),
  );
}

void main() {
  const ready = VoiceSettingsState(
    status: LoadStatus.ready,
    capability: DictationCapability(
      available: true,
      onDevice: true,
      permission: DictationPermission.granted,
      language: 'English',
    ),
  );
  const refused = VoiceSettingsState(
    status: LoadStatus.ready,
    settings: VoiceSettings(spokenCommands: false),
    capability: DictationCapability(
      available: false,
      onDevice: false,
      permission: DictationPermission.denied,
      language: 'Shona',
      reason: DictationUnavailableReason.notOnDevice,
    ),
  );
  final states = <String, VoiceSettingsState>{
    'ready': ready,
    'refused': refused,
  };
  final viewports = <KansoViewport>[
    KansoViewport.phone,
    ...KansoViewport.goldenSet,
  ];
  for (final brightness in Brightness.values) {
    group('settings, voice, ${brightness.name}', () {
      for (final viewport in viewports) {
        for (final MapEntry(key: label, value: seed) in states.entries) {
          testWidgets('$label at ${viewport.slug}', (tester) async {
            await expectGoldenOnBothPlatforms(
              tester,
              _scene(seed),
              name:
                  'voice_settings_${label}_${viewport.slug}_${brightness.name}',
              viewport: viewport,
              brightness: brightness,
            );
          });
        }
      }
    });
  }
}
