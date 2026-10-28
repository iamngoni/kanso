// Kanso — word_goal_intent_bloc_test.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/note/state/word_goal_intent/word_goal_intent_bloc.dart';

void main() {
  group('WordGoalIntentBloc', () {
    blocTest<WordGoalIntentBloc, WordGoalIntentState>(
      'a request is pending for its note',
      build: WordGoalIntentBloc.new,
      act: (bloc) => bloc.add(
        const WordGoalIntentEvent.requested(noteId: 'note:a'),
      ),
      expect: () => const <WordGoalIntentState>[
        WordGoalIntentState(request: 1, noteId: 'note:a'),
      ],
      verify: (bloc) => expect(bloc.state.pending, isTrue),
    );

    blocTest<WordGoalIntentBloc, WordGoalIntentState>(
      'consuming the current request clears it',
      build: WordGoalIntentBloc.new,
      act: (bloc) => bloc
        ..add(const WordGoalIntentEvent.requested(noteId: 'note:a'))
        ..add(const WordGoalIntentEvent.consumed(request: 1)),
      expect: () => const <WordGoalIntentState>[
        WordGoalIntentState(request: 1, noteId: 'note:a'),
        WordGoalIntentState(request: 1),
      ],
      verify: (bloc) => expect(bloc.state.pending, isFalse),
    );

    blocTest<WordGoalIntentBloc, WordGoalIntentState>(
      'consuming an older request leaves the current one pending',
      build: WordGoalIntentBloc.new,
      act: (bloc) => bloc
        ..add(const WordGoalIntentEvent.requested(noteId: 'note:a'))
        ..add(const WordGoalIntentEvent.consumed(request: 99)),
      expect: () => const <WordGoalIntentState>[
        WordGoalIntentState(request: 1, noteId: 'note:a'),
      ],
      verify: (bloc) => expect(bloc.state.pending, isTrue),
    );

    blocTest<WordGoalIntentBloc, WordGoalIntentState>(
      'a new request replaces the previous note target',
      build: WordGoalIntentBloc.new,
      act: (bloc) => bloc
        ..add(const WordGoalIntentEvent.requested(noteId: 'note:a'))
        ..add(const WordGoalIntentEvent.requested(noteId: 'note:b')),
      expect: () => const <WordGoalIntentState>[
        WordGoalIntentState(request: 1, noteId: 'note:a'),
        WordGoalIntentState(request: 2, noteId: 'note:b'),
      ],
      verify: (bloc) => expect(bloc.state.pending, isTrue),
    );

    blocTest<WordGoalIntentBloc, WordGoalIntentState>(
      'an empty note target is ignored',
      build: WordGoalIntentBloc.new,
      act: (bloc) => bloc.add(const WordGoalIntentEvent.requested(noteId: '')),
      expect: () => const <WordGoalIntentState>[],
      verify: (bloc) => expect(bloc.state.pending, isFalse),
    );
  });
}
