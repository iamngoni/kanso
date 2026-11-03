// Kanso — ask_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/ask/models/data/ask_outcome.dart';
import 'package:kanso/ask/models/data/ask_phase.dart';
import 'package:kanso/ask/state/ask/ask_bloc.dart';
import 'package:kanso/core/models/data/ai_note_ref.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/ask_receipt.dart';
import 'package:kanso/core/models/data/ask_scope.dart';
import 'package:kanso/core/models/data/ask_time_note.dart';
import 'package:kanso/core/models/data/ask_update.dart';
import 'package:kanso/core/models/data/load_status.dart';

import '../../../helpers/ai_blocs.dart';
import '../../../helpers/fixture_ai_repository.dart';

const _pricing = AiNoteRef(id: 'n1', title: 'Pricing');
const _plan = AiNoteRef(id: 'n2', title: 'Plan');

Future<void> _settle() => Future<void>.delayed(Duration.zero);

class RememberingAiRepository extends FixtureAiRepository {
  RememberingAiRepository() : super(settings: aiSettings(enabled: true));

  final List<String> conversation = <String>[];
  final List<List<String>> histories = <List<String>>[];

  @override
  Stream<AskUpdate> ask(
    String question, {
    AskScope scope = AskScope.everything,
    bool ignoreTime = false,
  }) {
    histories.add(List<String>.of(conversation));
    conversation.add(question);
    return const Stream<AskUpdate>.empty();
  }

  @override
  Future<Either<ApplicationError, Unit>> clear() async {
    conversation.clear();
    return super.clear();
  }
}

Future<(AskBloc, FixtureAiRepository)> _open({bool enabled = true}) async {
  final ai = FixtureAiRepository(settings: aiSettings(enabled: enabled));
  addTearDown(ai.close);
  final bloc = AskBloc(ai)..add(const AskEvent.started());
  addTearDown(bloc.close);
  await bloc.stream.firstWhere((s) => s.status == LoadStatus.ready);
  return (bloc, ai);
}

void main() {
  group('AskBloc', () {
    test(
      'opened from a notebook, it starts limited to that notebook',
      () async {
        const scope = AskScope(kind: 'book', value: 'b1', label: 'Work');
        final ai = FixtureAiRepository(settings: aiSettings(enabled: true));
        addTearDown(ai.close);
        final bloc = AskBloc(ai)..add(const AskEvent.started(scope: scope));
        addTearDown(bloc.close);
        await bloc.stream.firstWhere((s) => s.status == LoadStatus.ready);
        expect(bloc.state.scope, scope);
      },
    );

    test('opens knowing whether AI is on and where questions go', () async {
      final (bloc, _) = await _open();
      expect(bloc.state.aiEnabled, isTrue);
      expect(bloc.state.privacy, fixturePrivacy);
      expect(bloc.state.turns, isEmpty);
    });

    test('asks nothing while AI is off', () async {
      final (bloc, ai) = await _open(enabled: false);
      bloc.add(const AskEvent.questionAsked('What does it cost?'));
      await _settle();
      expect(ai.asked, isEmpty);
      expect(bloc.state.turns, isEmpty);
    });

    test('ignores a blank question', () async {
      final (bloc, ai) = await _open();
      bloc.add(const AskEvent.questionAsked('   '));
      await _settle();
      expect(ai.asked, isEmpty);
    });

    test('searching, thinking, notes, the answer as it grows, done', () async {
      final (bloc, ai) = await _open();
      bloc.add(const AskEvent.questionAsked(' What does it cost? '));
      await _settle();
      expect(ai.asked, <String>['What does it cost?']);
      expect(bloc.state.phase, AskPhase.searching);

      ai.askUpdates.add(const AskUpdate.thinking());
      await _settle();
      expect(bloc.state.phase, AskPhase.thinking);

      ai.askUpdates
        ..add(const AskUpdate.usedNotes(notes: <AiNoteRef>[_pricing]))
        ..add(const AskUpdate.answer(text: 'It costs'));
      await _settle();
      expect(bloc.state.phase, AskPhase.answering);
      expect(bloc.state.turns.single.answer, 'It costs');
      expect(bloc.state.busy, isTrue);

      ai.askUpdates.add(
        const AskUpdate.done(
          answer: 'It costs 40 [Pricing](kanso://open/note/n1).',
          used: <AiNoteRef>[_pricing, _plan],
        ),
      );
      await _settle();
      final turn = bloc.state.turns.single;
      expect(turn.outcome, AskOutcome.answered);
      expect(turn.used, <AiNoteRef>[_pricing, _plan]);
      expect(bloc.state.busy, isFalse);
    });

    test('a second question waits for the first to finish', () async {
      final (bloc, ai) = await _open();
      bloc.add(const AskEvent.questionAsked('one'));
      await _settle();
      bloc.add(const AskEvent.questionAsked('two'));
      await _settle();
      expect(ai.asked, <String>['one']);
    });

    test('a question is asked over the scope that was chosen', () async {
      final (bloc, ai) = await _open();
      const work = AskScope(kind: 'book', value: 'b1', label: 'Work');
      ai.scopes = const <AskScope>[AskScope.everything, work];
      bloc.add(const AskEvent.scopesRequested());
      await _settle();
      expect(bloc.state.scopes, ai.scopes);
      bloc
        ..add(const AskEvent.scopeChosen(work))
        ..add(const AskEvent.questionAsked('what?'));
      await _settle();
      expect(bloc.state.turns.single.scope, work);
      ai.askUpdates.add(const AskUpdate.nothingMatches(narrow: true));
      await _settle();
      expect(bloc.state.turns.single.narrow, isTrue);
      bloc.add(const AskEvent.searchedAll());
      await _settle();
      expect(bloc.state.scope, AskScope.everything);
      expect(bloc.state.turns.single.question, 'what?');
      expect(bloc.state.turns.single.scope, AskScope.everything);
    });

    test(
      'receipts, the time and the banner reach the turn; save works',
      () async {
        final (bloc, ai) = await _open();
        bloc.add(const AskEvent.questionAsked('last month?'));
        await _settle();
        ai.askUpdates
          ..add(const AskUpdate.time(time: AskTimeNote(label: 'Written June')))
          ..add(
            const AskUpdate.done(
              answer: 'Yes [1](kanso-cite:1)',
              used: <AiNoteRef>[_pricing],
              receipts: <AskReceipt>[AskReceipt(n: 1, noteId: 'n1', end: 2)],
              banner: 'Careful.',
              unverified: 1,
            ),
          );
        await _settle();
        final turn = bloc.state.turns.single;
        expect(turn.time?.label, 'Written June');
        expect(turn.receipts.single.noteId, 'n1');
        expect(turn.banner, 'Careful.');
        expect(turn.unverified, 1);
        bloc
          ..add(const AskEvent.answerSaved())
          ..add(const AskEvent.answerSaved());
        await _settle();
        expect(ai.saves, 1, reason: 'a saved answer is saved once');
        expect(bloc.state.turns.single.savedNoteId, ai.savedId);
      },
    );

    test('a follow-up joins the conversation', () async {
      final (bloc, ai) = await _open();
      bloc.add(const AskEvent.questionAsked('one'));
      await _settle();
      ai.askUpdates.add(const AskUpdate.done(answer: 'A', used: <AiNoteRef>[]));
      await _settle();
      bloc.add(const AskEvent.questionAsked('two'));
      await _settle();
      expect(bloc.state.turns.map((t) => t.question), <String>['one', 'two']);
    });

    test('nothing matches, cut off and an error end the question', () async {
      final (bloc, ai) = await _open();
      for (final (update, outcome) in <(AskUpdate, AskOutcome)>[
        (const AskUpdate.nothingMatches(), AskOutcome.nothingMatches),
        (const AskUpdate.cutOff(), AskOutcome.cutOff),
        (const AskUpdate.error(message: 'No.'), AskOutcome.failed),
      ]) {
        bloc.add(const AskEvent.questionAsked('q'));
        await _settle();
        ai.askUpdates.add(update);
        await _settle();
        expect(bloc.state.turns.last.outcome, outcome);
        expect(bloc.state.busy, isFalse);
      }
      expect(bloc.state.turns.last.failure, 'No.');
    });

    test('Stop keeps what was written and tells the core', () async {
      final (bloc, ai) = await _open();
      bloc.add(const AskEvent.questionAsked('q'));
      await _settle();
      ai.askUpdates.add(const AskUpdate.answer(text: 'Half'));
      await _settle();
      bloc.add(const AskEvent.stopped());
      await _settle();
      expect(bloc.state.turns.single.outcome, AskOutcome.stopped);
      expect(bloc.state.turns.single.answer, 'Half');
      expect(bloc.state.busy, isFalse);
      expect(ai.stops, 1);
      // What arrives after is not applied.
      ai.askUpdates.add(const AskUpdate.answer(text: 'Half and more'));
      await _settle();
      expect(bloc.state.turns.single.answer, 'Half');
    });

    test('Clear forgets the conversation and stops what is running', () async {
      final (bloc, ai) = await _open();
      bloc.add(const AskEvent.questionAsked('q'));
      await _settle();
      bloc.add(const AskEvent.cleared());
      await _settle();
      expect(bloc.state.turns, isEmpty);
      expect(bloc.state.busy, isFalse);
      expect(ai.clears, 1);
    });

    test('a failed question can be tried again', () async {
      final (bloc, ai) = await _open();
      bloc.add(const AskEvent.questionAsked('q'));
      await _settle();
      ai.askUpdates.add(const AskUpdate.error(message: 'No.'));
      await _settle();
      bloc.add(const AskEvent.retried());
      await _settle();
      expect(ai.asked, <String>['q', 'q']);
      expect(bloc.state.turns, hasLength(1));
      expect(bloc.state.busy, isTrue);
    });

    test('updates that end with no word are an answer or a failure', () async {
      final (bloc, ai) = await _open();
      bloc.add(const AskEvent.questionAsked('q'));
      await _settle();
      bloc.add(const AskEvent.streamEnded());
      await _settle();
      expect(bloc.state.turns.single.outcome, AskOutcome.failed);

      bloc.add(const AskEvent.questionAsked('r'));
      await _settle();
      ai.askUpdates.add(const AskUpdate.answer(text: 'Some'));
      await _settle();
      bloc.add(const AskEvent.streamEnded());
      await _settle();
      expect(bloc.state.turns.last.outcome, AskOutcome.answered);
    });

    test('AI turned on elsewhere is seen', () async {
      final (bloc, ai) = await _open(enabled: false);
      expect(bloc.state.aiEnabled, isFalse);
      ai.changes.add(aiSettings(enabled: true));
      await bloc.stream.firstWhere((s) => s.aiEnabled);
    });

    test('closing stops the question being answered', () async {
      final (bloc, ai) = await _open();
      bloc.add(const AskEvent.questionAsked('q'));
      await _settle();
      await bloc.close();
      expect(ai.stops, 1);
    });

    test('reopening starts a new repository conversation', () async {
      final ai = RememberingAiRepository();
      final first = AskBloc(ai)..add(const AskEvent.started());
      await first.stream.firstWhere((s) => s.status == LoadStatus.ready);
      first.add(const AskEvent.questionAsked('first question'));
      await _settle();
      await first.close();

      final second = AskBloc(ai)..add(const AskEvent.started());
      await second.stream.firstWhere((s) => s.status == LoadStatus.ready);
      second.add(const AskEvent.questionAsked('second question'));
      await _settle();

      expect(ai.histories, <List<String>>[<String>[], <String>[]]);
      await second.close();
      await ai.close();
    });
  });
}
