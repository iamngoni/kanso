// Kanso — assist_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/assist/models/data/assist_status.dart';
import 'package:kanso/assist/state/assist/assist_bloc.dart';
import 'package:kanso/core/models/data/assist_action.dart';
import 'package:kanso/core/models/data/assist_update.dart';

import '../../../helpers/fixture_ai_repository.dart';

Future<void> _settle() => Future<void>.delayed(Duration.zero);

(AssistBloc, FixtureAiRepository) _make() {
  final ai = FixtureAiRepository();
  addTearDown(ai.close);
  final bloc = AssistBloc(ai);
  addTearDown(bloc.close);
  return (bloc, ai);
}

AssistEvent _improve() => const AssistEvent.requested(
  noteId: 'n1',
  action: AssistAction.improve,
  before: 'Earlier.',
  selection: 'we ship on friday',
);

void main() {
  group('AssistBloc', () {
    test('lists the actions as the core words them', () {
      final (bloc, _) = _make();
      expect(bloc.actions.map((a) => a.label), contains('Improve Writing'));
    });

    test('streams a suggestion, then offers it', () async {
      final (bloc, ai) = _make();
      bloc.add(_improve());
      await _settle();
      expect(bloc.state.status, AssistStatus.running);
      ai.assistUpdates.add(const AssistUpdate.text(text: 'We ship'));
      await _settle();
      expect(bloc.state.suggestion, 'We ship');
      ai.assistUpdates.add(
        const AssistUpdate.done(suggestion: ' We ship on Friday.'),
      );
      await _settle();
      expect(bloc.state.status, AssistStatus.ready);
      expect(bloc.state.suggestion, 'We ship on Friday.');
    });

    test('an empty answer is a failure', () async {
      final (bloc, ai) = _make();
      bloc.add(_improve());
      await _settle();
      ai.assistUpdates.add(const AssistUpdate.done(suggestion: '  '));
      await _settle();
      expect(bloc.state.status, AssistStatus.failed);
      expect(bloc.state.failure, isNull);
    });

    test('a note kept out of AI says so', () async {
      final (bloc, ai) = _make();
      bloc.add(_improve());
      await _settle();
      ai.assistUpdates.add(const AssistUpdate.keptOut());
      await _settle();
      expect(bloc.state.status, AssistStatus.keptOut);
    });

    test('an error is kept in the core’s words', () async {
      final (bloc, ai) = _make();
      bloc.add(_improve());
      await _settle();
      ai.assistUpdates.add(const AssistUpdate.error(message: 'Offline.'));
      await _settle();
      expect(bloc.state.status, AssistStatus.failed);
      expect(bloc.state.failure, 'Offline.');
    });

    test('Stop keeps what arrived and ignores the rest', () async {
      final (bloc, ai) = _make();
      bloc.add(_improve());
      await _settle();
      ai.assistUpdates.add(const AssistUpdate.text(text: 'We'));
      await _settle();
      bloc.add(const AssistEvent.stopped());
      await _settle();
      expect(bloc.state.status, AssistStatus.stopped);
      ai.assistUpdates.add(const AssistUpdate.text(text: 'We ship'));
      await _settle();
      expect(bloc.state.suggestion, 'We');
    });

    test('Discard goes back to the start', () async {
      final (bloc, ai) = _make();
      bloc.add(_improve());
      await _settle();
      ai.assistUpdates.add(const AssistUpdate.done(suggestion: 'Done.'));
      await _settle();
      bloc.add(const AssistEvent.discarded());
      await _settle();
      expect(bloc.state, const AssistState());
    });

    test('a summary of the whole note streams the same way', () async {
      final (bloc, ai) = _make();
      bloc.add(const AssistEvent.summaryRequested('n1'));
      await _settle();
      expect(bloc.state.wholeNote, isTrue);
      ai.assistUpdates.add(const AssistUpdate.done(suggestion: 'Short.'));
      await _settle();
      expect(bloc.state.status, AssistStatus.ready);
    });

    test('updates that end with no word are a failure', () async {
      final (bloc, _) = _make();
      bloc.add(_improve());
      await _settle();
      bloc.add(const AssistEvent.streamEnded());
      await _settle();
      expect(bloc.state.status, AssistStatus.failed);
    });
  });
}
