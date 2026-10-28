// Kanso — note_assist_intent_bloc_test.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/assist_action.dart';
import 'package:kanso/note/state/note_assist_intent/note_assist_intent_bloc.dart';

void main() {
  group('NoteAssistIntentBloc', () {
    blocTest<NoteAssistIntentBloc, NoteAssistIntentState>(
      'a request is pending with its note and action',
      build: NoteAssistIntentBloc.new,
      act: (bloc) => bloc.add(
        const NoteAssistIntentEvent.requested(
          noteId: 'note:a',
          action: AssistAction.improve,
        ),
      ),
      expect: () => const <NoteAssistIntentState>[
        NoteAssistIntentState(
          request: 1,
          noteId: 'note:a',
          action: AssistAction.improve,
        ),
      ],
    );

    blocTest<NoteAssistIntentBloc, NoteAssistIntentState>(
      'consuming the current request clears it',
      build: NoteAssistIntentBloc.new,
      act: (bloc) => bloc
        ..add(
          const NoteAssistIntentEvent.requested(
            noteId: 'note:a',
            summarizeNote: true,
          ),
        )
        ..add(const NoteAssistIntentEvent.consumed(request: 1)),
      expect: () => const <NoteAssistIntentState>[
        NoteAssistIntentState(
          request: 1,
          noteId: 'note:a',
          summarizeNote: true,
        ),
        NoteAssistIntentState(request: 1),
      ],
    );

    blocTest<NoteAssistIntentBloc, NoteAssistIntentState>(
      'consuming an older request leaves the current one pending',
      build: NoteAssistIntentBloc.new,
      act: (bloc) => bloc
        ..add(
          const NoteAssistIntentEvent.requested(
            noteId: 'note:a',
            action: AssistAction.shorten,
          ),
        )
        ..add(const NoteAssistIntentEvent.consumed(request: 99)),
      expect: () => const <NoteAssistIntentState>[
        NoteAssistIntentState(
          request: 1,
          noteId: 'note:a',
          action: AssistAction.shorten,
        ),
      ],
    );

    blocTest<NoteAssistIntentBloc, NoteAssistIntentState>(
      'a new request replaces the previous note target',
      build: NoteAssistIntentBloc.new,
      act: (bloc) => bloc
        ..add(
          const NoteAssistIntentEvent.requested(
            noteId: 'note:a',
            action: AssistAction.improve,
          ),
        )
        ..add(
          const NoteAssistIntentEvent.requested(
            noteId: 'note:b',
            action: AssistAction.fix,
          ),
        ),
      expect: () => const <NoteAssistIntentState>[
        NoteAssistIntentState(
          request: 1,
          noteId: 'note:a',
          action: AssistAction.improve,
        ),
        NoteAssistIntentState(
          request: 2,
          noteId: 'note:b',
          action: AssistAction.fix,
        ),
      ],
    );

    blocTest<NoteAssistIntentBloc, NoteAssistIntentState>(
      'an empty request is ignored',
      build: NoteAssistIntentBloc.new,
      act: (bloc) => bloc.add(
        const NoteAssistIntentEvent.requested(noteId: 'note:a'),
      ),
      expect: () => const <NoteAssistIntentState>[],
    );
  });
}
