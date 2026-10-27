// Kanso — default_notebook_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/core/models/repos/abs/preferences_repository.dart';
import 'package:kanso/settings/state/default_notebook/default_notebook_bloc.dart';
import 'package:mocktail/mocktail.dart';

class MockPreferences extends Mock implements PreferencesRepository {}

void main() {
  late MockPreferences preferences;
  late Preferences saved;

  setUpAll(() {
    registerFallbackValue((Preferences current) => current);
  });

  setUp(() {
    preferences = MockPreferences();
    saved = const Preferences();
    when(
      preferences.load,
    ).thenAnswer((_) async => Right<ApplicationError, Preferences>(saved));
    when(() => preferences.update(any())).thenAnswer((call) async {
      final change =
          call.positionalArguments.first as Preferences Function(Preferences);
      saved = change(saved);
      return Right<ApplicationError, Preferences>(saved);
    });
  });

  DefaultNotebookBloc build() => DefaultNotebookBloc(preferences);

  group('DefaultNotebookBloc', () {
    test('starts on the first notebook', () {
      expect(build().state.notebookId, isNull);
    });

    blocTest<DefaultNotebookBloc, DefaultNotebookState>(
      'started reads the notebook that was chosen',
      setUp: () => saved = const Preferences(defaultNotebookId: 'book:a'),
      build: build,
      act: (bloc) => bloc.add(const DefaultNotebookEvent.started()),
      expect: () => const <DefaultNotebookState>[
        DefaultNotebookState(notebookId: 'book:a'),
      ],
    );

    blocTest<DefaultNotebookBloc, DefaultNotebookState>(
      'started with preferences that cannot be read is the first notebook',
      setUp: () {
        when(preferences.load).thenAnswer(
          (_) async => const Left<ApplicationError, Preferences>(
            ApplicationError.storage(message: 'Unreadable.'),
          ),
        );
      },
      build: build,
      act: (bloc) => bloc.add(const DefaultNotebookEvent.started()),
      expect: () => const <DefaultNotebookState>[DefaultNotebookState()],
    );

    blocTest<DefaultNotebookBloc, DefaultNotebookState>(
      'chosen shows the notebook at once and saves it, and null goes back to '
      'the first',
      build: build,
      act: (bloc) async {
        bloc.add(const DefaultNotebookEvent.chosen('book:b'));
        await bloc.stream.first;
        await Future<void>.delayed(Duration.zero);
        expect(saved.defaultNotebookId, 'book:b');
        bloc.add(const DefaultNotebookEvent.chosen(null));
      },
      expect: () => const <DefaultNotebookState>[
        DefaultNotebookState(notebookId: 'book:b'),
        DefaultNotebookState(),
      ],
      verify: (_) => expect(saved.defaultNotebookId, isNull),
    );
  });
}
