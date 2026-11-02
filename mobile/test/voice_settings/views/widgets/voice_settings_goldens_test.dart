// Kanso — voice_settings_goldens_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/ai_settings/state/ai_status/ai_status_bloc.dart';
import 'package:kanso/core/models/data/ai_settings.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:kanso/dictation/models/data/dictation_capability.dart';
import 'package:kanso/dictation/models/data/dictation_permission.dart';
import 'package:kanso/dictation/models/data/dictation_unavailable_reason.dart';
import 'package:kanso/voice_capture/state/voice_drops/voice_drops_bloc.dart';
import 'package:kanso/voice_settings/models/data/voice_settings.dart';
import 'package:kanso/voice_settings/models/data/voice_tidy_mode.dart';
import 'package:kanso/voice_settings/state/voice_settings/voice_settings_bloc.dart';
import 'package:kanso/voice_settings/views/widgets/voice_settings_pane.dart';

import '../../../dictation/helpers/mock_dictation_repository.dart';
import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/fixture_ai_repository.dart';
import '../../../helpers/golden.dart';
import '../../../helpers/kanso_viewport.dart';
import '../../../helpers/mock_repositories.dart';
import '../../helpers/mock_voice_settings_repository.dart';

class SeededVoiceSettingsBloc extends VoiceSettingsBloc {
  SeededVoiceSettingsBloc(VoiceSettingsState seed)
    : super(MockVoiceSettingsRepository(), SpokenRepository().repository) {
    emit(seed);
  }
}

class SeededVoiceDropsBloc extends VoiceDropsBloc {
  SeededVoiceDropsBloc(VoiceDropsState seed)
    : super(
        MockVoiceCaptureRepository(),
        SpokenRepository().repository,
        MockVoiceSettingsRepository(),
      ) {
    emit(seed);
  }
}

Widget _scene(
  VoiceSettingsState seed, {
  bool ai = false,
  VoiceDropsState drops = const VoiceDropsState(),
}) {
  final bloc = SeededVoiceSettingsBloc(seed);
  closeBlocLater(bloc);
  final status = AiStatusBloc(
    FixtureAiRepository(
      settings: ai
          ? const AiSettings(enabled: true, provider: 'anthropic')
          : const AiSettings(),
    ),
  )..add(const AiStatusEvent.started());
  closeBlocLater(status);
  final waiting = SeededVoiceDropsBloc(drops);
  closeBlocLater(waiting);
  return MultiBlocProvider(
    providers: <BlocProvider<dynamic>>[
      BlocProvider<VoiceSettingsBloc>.value(value: bloc),
      BlocProvider<AiStatusBloc>.value(value: status),
      BlocProvider<VoiceDropsBloc>.value(value: waiting),
    ],
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
  const tidyAlways = VoiceSettingsState(
    status: LoadStatus.ready,
    settings: VoiceSettings(tidy: VoiceTidyMode.always, keepAudio: true),
    capability: DictationCapability(
      available: true,
      onDevice: true,
      permission: DictationPermission.granted,
      language: 'English',
    ),
  );
  final states = <String, Widget Function()>{
    'ready': () => _scene(ready),
    'refused': () => _scene(refused),
    'ai_on': () => _scene(tidyAlways, ai: true),
    'waiting': () => _scene(
      ready,
      drops: const VoiceDropsState(
        waiting: 2,
        problems: <String>['The recording is over 10 MB.'],
      ),
    ),
  };
  final viewports = <KansoViewport>[
    KansoViewport.phone,
    ...KansoViewport.goldenSet,
  ];
  for (final brightness in Brightness.values) {
    group('settings, voice, ${brightness.name}', () {
      for (final viewport in viewports) {
        for (final MapEntry(key: label, value: scene) in states.entries) {
          testWidgets('$label at ${viewport.slug}', (tester) async {
            await expectGoldenOnBothPlatforms(
              tester,
              scene(),
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
