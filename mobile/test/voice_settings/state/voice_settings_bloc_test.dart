// Kanso — voice_settings_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:kanso/dictation/models/data/dictation_capability.dart';
import 'package:kanso/dictation/models/data/dictation_permission.dart';
import 'package:kanso/voice_settings/models/data/voice_settings.dart';
import 'package:kanso/voice_settings/state/voice_settings/voice_settings_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../dictation/helpers/mock_dictation_repository.dart';
import '../helpers/mock_voice_settings_repository.dart';

const _capability = DictationCapability(
  available: true,
  onDevice: true,
  permission: DictationPermission.granted,
  language: 'English',
);

void main() {
  late MockVoiceSettingsRepository settings;
  late SpokenRepository spoken;

  setUpAll(() => registerFallbackValue(const VoiceSettings()));

  setUp(() {
    settings = MockVoiceSettingsRepository();
    spoken = SpokenRepository(capability: _capability);
    when(settings.settings).thenAnswer(
      (_) async => const Right<ApplicationError, VoiceSettings>(
        VoiceSettings(),
      ),
    );
    when(() => settings.save(any())).thenAnswer(
      (invocation) async => Right<ApplicationError, VoiceSettings>(
        invocation.positionalArguments.first as VoiceSettings,
      ),
    );
  });

  tearDown(() => spoken.close());

  blocTest<VoiceSettingsBloc, VoiceSettingsState>(
    'reads the choices and what the recognizer can do',
    build: () => VoiceSettingsBloc(settings, spoken.repository),
    act: (bloc) => bloc.add(const VoiceSettingsEvent.started()),
    expect: () => <VoiceSettingsState>[
      const VoiceSettingsState(
        status: LoadStatus.ready,
        capability: _capability,
      ),
    ],
  );

  blocTest<VoiceSettingsBloc, VoiceSettingsState>(
    'fails when the choices cannot be read, and still says what it can do',
    setUp: () => when(settings.settings).thenAnswer(
      (_) async => const Left<ApplicationError, VoiceSettings>(
        ApplicationError.storage(message: 'x'),
      ),
    ),
    build: () => VoiceSettingsBloc(settings, spoken.repository),
    act: (bloc) => bloc.add(const VoiceSettingsEvent.started()),
    expect: () => <VoiceSettingsState>[
      const VoiceSettingsState(
        status: LoadStatus.failed,
        capability: _capability,
      ),
    ],
  );

  blocTest<VoiceSettingsBloc, VoiceSettingsState>(
    'switches spoken commands off, and keeps the choice',
    build: () => VoiceSettingsBloc(settings, spoken.repository),
    act: (bloc) => bloc.add(
      const VoiceSettingsEvent.spokenCommandsChanged(on: false),
    ),
    expect: () => <VoiceSettingsState>[
      const VoiceSettingsState(settings: VoiceSettings(spokenCommands: false)),
    ],
    verify: (_) => verify(
      () => settings.save(const VoiceSettings(spokenCommands: false)),
    ).called(1),
  );

  blocTest<VoiceSettingsBloc, VoiceSettingsState>(
    'moves the switch back when the choice was not kept',
    setUp: () => when(() => settings.save(any())).thenAnswer(
      (_) async => const Left<ApplicationError, VoiceSettings>(
        ApplicationError.storage(message: 'x'),
      ),
    ),
    build: () => VoiceSettingsBloc(settings, spoken.repository),
    act: (bloc) => bloc.add(
      const VoiceSettingsEvent.spokenCommandsChanged(on: false),
    ),
    expect: () => <VoiceSettingsState>[
      const VoiceSettingsState(settings: VoiceSettings(spokenCommands: false)),
      const VoiceSettingsState(),
    ],
  );

  blocTest<VoiceSettingsBloc, VoiceSettingsState>(
    'opens the phone settings when asked',
    build: () => VoiceSettingsBloc(settings, spoken.repository),
    act: (bloc) => bloc.add(const VoiceSettingsEvent.systemSettingsRequested()),
    expect: () => <VoiceSettingsState>[],
    verify: (_) => verify(spoken.repository.openSettings).called(1),
  );
}
