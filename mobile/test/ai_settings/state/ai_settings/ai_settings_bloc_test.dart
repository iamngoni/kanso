// Kanso — ai_settings_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/ai_settings/models/data/ai_key.dart';
import 'package:kanso/ai_settings/models/data/ai_test_status.dart';
import 'package:kanso/ai_settings/state/ai_settings/ai_settings_bloc.dart';
import 'package:kanso/core/models/data/ai_settings.dart';
import 'package:kanso/core/models/data/ai_settings_change.dart';
import 'package:kanso/core/models/data/ai_test_result.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/ai_blocs.dart';
import '../../../helpers/fixture_ai_repository.dart';
import '../../../helpers/mock_repositories.dart';

Either<ApplicationError, AiSettings> _ok(AiSettings settings) =>
    Right<ApplicationError, AiSettings>(settings);

void main() {
  setUpAll(() {
    registerFallbackValue(
      const AiSettingsChange(enabled: false, provider: ''),
    );
  });

  group('AiSettingsBloc', () {
    blocTest<AiSettingsBloc, AiSettingsState>(
      'reads the settings and the providers',
      build: () => AiSettingsBloc(FixtureAiRepository()),
      act: (bloc) => bloc.add(const AiSettingsEvent.started()),
      expect: () => <Matcher>[
        isA<AiSettingsState>().having(
          (s) => s.status,
          'status',
          LoadStatus.loading,
        ),
        isA<AiSettingsState>()
            .having((s) => s.status, 'status', LoadStatus.ready)
            .having((s) => s.providers, 'providers', fixtureProviders)
            .having((s) => s.provider?.label, 'provider', 'Anthropic'),
      ],
    );

    blocTest<AiSettingsBloc, AiSettingsState>(
      'fails when the settings cannot be read',
      setUp: () {},
      build: () {
        final ai = MockAiRepository();
        when(ai.settings).thenAnswer(
          (_) async => const Left<ApplicationError, AiSettings>(
            ApplicationError.storage(message: 'x'),
          ),
        );
        when(ai.providers).thenAnswer(
          (_) async => const Right(fixtureProviders),
        );
        return AiSettingsBloc(ai);
      },
      act: (bloc) => bloc.add(const AiSettingsEvent.started()),
      skip: 1,
      expect: () => <Matcher>[
        isA<AiSettingsState>().having(
          (s) => s.status,
          'status',
          LoadStatus.failed,
        ),
      ],
    );

    test('turning AI on saves it with the rest unchanged', () async {
      final ai = FixtureAiRepository(settings: aiSettings());
      final bloc = AiSettingsBloc(ai)..add(const AiSettingsEvent.started());
      await bloc.stream.firstWhere((s) => s.status == LoadStatus.ready);
      bloc.add(const AiSettingsEvent.enabledChanged(enabled: true));
      await bloc.stream.firstWhere((s) => s.settings.enabled);
      expect(ai.saved.single.enabled, isTrue);
      expect(ai.saved.single.provider, 'anthropic');
      await bloc.close();
    });

    test('a different provider starts from its own defaults', () async {
      final ai = FixtureAiRepository(
        settings: aiSettings().copyWith(baseUrl: 'https://x', model: 'm'),
      );
      final bloc = AiSettingsBloc(ai)..add(const AiSettingsEvent.started());
      await bloc.stream.firstWhere((s) => s.status == LoadStatus.ready);
      bloc.add(const AiSettingsEvent.providerChosen('ollama'));
      await bloc.stream.firstWhere((s) => s.settings.provider == 'ollama');
      expect(ai.saved.single.baseUrl, isEmpty);
      expect(ai.saved.single.model, isEmpty);
      await bloc.close();
    });

    test('switching providers discards a pending field edit', () async {
      final ai = FixtureAiRepository(settings: aiSettings());
      final bloc = AiSettingsBloc(ai)..add(const AiSettingsEvent.started());
      await bloc.stream.firstWhere((s) => s.status == LoadStatus.ready);
      bloc
        ..add(
          const AiSettingsEvent.fieldsChanged(
            AiFields(address: 'https://old-provider.example', model: 'old'),
          ),
        )
        ..add(const AiSettingsEvent.providerChosen('ollama'));
      await bloc.stream.firstWhere((s) => s.settings.provider == 'ollama');
      await Future<void>.delayed(const Duration(milliseconds: 550));

      expect(ai.saved, hasLength(1));
      expect(ai.saved.single.provider, 'ollama');
      expect(ai.saved.single.baseUrl, isEmpty);
      expect(ai.saved.single.model, isEmpty);
      await bloc.close();
    });

    test('typing is saved once, after a pause', () async {
      final ai = FixtureAiRepository(settings: aiSettings());
      final bloc = AiSettingsBloc(ai)..add(const AiSettingsEvent.started());
      await bloc.stream.firstWhere((s) => s.status == LoadStatus.ready);
      bloc
        ..add(
          const AiSettingsEvent.fieldsChanged(AiFields(address: 'http://a')),
        )
        ..add(
          const AiSettingsEvent.fieldsChanged(
            AiFields(address: ' http://ab ', model: 'm'),
          ),
        );
      await bloc.stream.firstWhere((s) => s.settings.model == 'm');
      expect(ai.saved, hasLength(1));
      expect(ai.saved.single.baseUrl, 'http://ab');
      await bloc.close();
    });

    test('a saved key is only ever "saved": it is not in the state', () async {
      final ai = FixtureAiRepository(settings: aiSettings());
      final bloc = AiSettingsBloc(ai)..add(const AiSettingsEvent.started());
      await bloc.stream.firstWhere((s) => s.status == LoadStatus.ready);
      const secret = 'sk-test-0123456789';
      bloc.add(const AiSettingsEvent.keySaved(AiKey(secret)));
      final after = await bloc.stream.firstWhere((s) => s.settings.hasKey);
      expect(ai.keysSet, <String>[secret]);
      expect(after.toString(), isNot(contains(secret)));
      expect(const AiKey(secret).toString(), isNot(contains(secret)));
      expect(
        const AiSettingsEvent.keySaved(AiKey(secret)).toString(),
        isNot(contains(secret)),
      );
      bloc.add(const AiSettingsEvent.keyRemoved());
      await bloc.stream.firstWhere((s) => !s.settings.hasKey);
      expect(ai.keysSet.last, isEmpty);
      await bloc.close();
    });

    test('Test saves what is typed, then shows what the model said', () async {
      final ai = FixtureAiRepository(settings: aiSettings());
      final bloc = AiSettingsBloc(ai)..add(const AiSettingsEvent.started());
      await bloc.stream.firstWhere((s) => s.status == LoadStatus.ready);
      bloc.add(
        const AiSettingsEvent.testRequested(AiFields(model: 'big')),
      );
      final done = await bloc.stream.firstWhere(
        (s) => s.test == AiTestStatus.ok,
      );
      expect(done.testMessage, 'OK');
      expect(ai.saved.single.model, 'big');
      await bloc.close();
    });

    test('Test shows why it failed, in the core’s words', () async {
      final ai = FixtureAiRepository(settings: aiSettings())
        ..testResult = const AiTestResult.failed(
          message: 'The key was rejected.',
        );
      final bloc = AiSettingsBloc(ai)..add(const AiSettingsEvent.started());
      await bloc.stream.firstWhere((s) => s.status == LoadStatus.ready);
      bloc.add(const AiSettingsEvent.testRequested(AiFields()));
      final done = await bloc.stream.firstWhere(
        (s) => s.test == AiTestStatus.failed,
      );
      expect(done.testMessage, 'The key was rejected.');
      // Changing a setting forgets the result.
      bloc.add(const AiSettingsEvent.enabledChanged(enabled: true));
      final cleared = await bloc.stream.firstWhere(
        (s) => s.test == AiTestStatus.idle,
      );
      expect(cleared.testMessage, isEmpty);
      await bloc.close();
    });

    test('a change that cannot be saved is said', () async {
      final ai = MockAiRepository();
      when(ai.settings).thenAnswer((_) async => _ok(aiSettings()));
      when(ai.providers).thenAnswer((_) async => const Right(fixtureProviders));
      when(() => ai.saveSettings(any())).thenAnswer(
        (_) async => const Left<ApplicationError, AiSettings>(
          ApplicationError.storage(message: 'x'),
        ),
      );
      final bloc = AiSettingsBloc(ai)..add(const AiSettingsEvent.started());
      await bloc.stream.firstWhere((s) => s.status == LoadStatus.ready);
      bloc.add(const AiSettingsEvent.enabledChanged(enabled: true));
      final after = await bloc.stream.firstWhere((s) => s.error != null);
      expect(after.settings.enabled, isFalse);
      await bloc.close();
    });
  });
}
