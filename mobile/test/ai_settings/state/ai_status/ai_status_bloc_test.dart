// Kanso — ai_status_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/ai_settings/state/ai_status/ai_status_bloc.dart';
import 'package:kanso/core/models/data/ai_settings.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/mock_repositories.dart';

void main() {
  late MockAiRepository ai;
  late StreamController<AiSettings> changes;

  setUp(() {
    ai = MockAiRepository();
    changes = StreamController<AiSettings>.broadcast();
    addTearDown(changes.close);
    when(() => ai.settingsChanges()).thenAnswer((_) => changes.stream);
  });

  group('AiStatusBloc', () {
    test('starts with AI off and nothing read', () {
      final state = AiStatusBloc(ai).state;
      expect(state.enabled, isFalse);
      expect(state.loaded, isFalse);
    });

    blocTest<AiStatusBloc, AiStatusState>(
      'reads the settings, then follows them',
      setUp: () => when(() => ai.settings()).thenAnswer(
        (_) async => const Right<ApplicationError, AiSettings>(AiSettings()),
      ),
      build: () => AiStatusBloc(ai),
      act: (bloc) async {
        bloc.add(const AiStatusEvent.started());
        await Future<void>.delayed(Duration.zero);
        changes.add(const AiSettings(enabled: true));
      },
      expect: () => <Matcher>[
        isA<AiStatusState>()
            .having((s) => s.loaded, 'loaded', isTrue)
            .having((s) => s.enabled, 'enabled', isFalse),
        isA<AiStatusState>().having((s) => s.enabled, 'enabled', isTrue),
      ],
    );

    blocTest<AiStatusBloc, AiStatusState>(
      'leaves AI off when the settings cannot be read',
      setUp: () => when(() => ai.settings()).thenAnswer(
        (_) async => const Left<ApplicationError, AiSettings>(
          ApplicationError.storage(message: 'x'),
        ),
      ),
      build: () => AiStatusBloc(ai),
      act: (bloc) => bloc.add(const AiStatusEvent.started()),
      expect: () => <Matcher>[
        isA<AiStatusState>()
            .having((s) => s.loaded, 'loaded', isTrue)
            .having((s) => s.enabled, 'enabled', isFalse),
      ],
    );
  });
}
