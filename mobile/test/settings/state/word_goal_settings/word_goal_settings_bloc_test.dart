// Kanso — word_goal_settings_bloc_test.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:kanso/settings/state/word_goal_settings/word_goal_settings_bloc.dart';

import '../../../helpers/fixture_library_repository.dart';

void main() {
  late FixtureLibraryRepository library;

  setUp(() => library = FixtureLibraryRepository());

  WordGoalSettingsBloc build() => WordGoalSettingsBloc(library);

  group('WordGoalSettingsBloc', () {
    blocTest<WordGoalSettingsBloc, WordGoalSettingsState>(
      'started reads the library default',
      build: () {
        library.wordGoalDefault = 1000;
        return build();
      },
      act: (bloc) => bloc.add(const WordGoalSettingsEvent.started()),
      expect: () => const <WordGoalSettingsState>[
        WordGoalSettingsState(status: LoadStatus.ready, words: 1000),
      ],
    );

    blocTest<WordGoalSettingsBloc, WordGoalSettingsState>(
      'clearing the default persists no goal',
      build: build,
      seed: () => const WordGoalSettingsState(
        status: LoadStatus.ready,
        words: 500,
      ),
      act: (bloc) => bloc.add(
        const WordGoalSettingsEvent.defaultChanged(null),
      ),
      expect: () => const <WordGoalSettingsState>[
        WordGoalSettingsState(status: LoadStatus.ready),
      ],
      verify: (_) => expect(library.wordGoalDefault, isNull),
    );

    blocTest<WordGoalSettingsBloc, WordGoalSettingsState>(
      'a failed change restores the saved default and says why',
      build: build,
      seed: () => const WordGoalSettingsState(
        status: LoadStatus.ready,
        words: 500,
      ),
      act: (bloc) async {
        library.wordGoalError = const ApplicationError.storage(
          message: 'Cannot write settings.',
        );
        bloc.add(const WordGoalSettingsEvent.defaultChanged(1000));
      },
      expect: () => const <WordGoalSettingsState>[
        WordGoalSettingsState(status: LoadStatus.ready, words: 1000),
        WordGoalSettingsState(
          status: LoadStatus.ready,
          words: 500,
          error: ApplicationError.storage(message: 'Cannot write settings.'),
        ),
      ],
      verify: (_) => expect(library.wordGoalDefault, isNull),
    );
  });
}
