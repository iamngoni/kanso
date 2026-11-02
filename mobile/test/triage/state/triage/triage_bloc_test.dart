// Kanso — triage_bloc_test.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/triage_types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:kanso/core/state/undo_notice/undo_notice_bloc.dart';
import 'package:kanso/triage/models/data/triage_card_state.dart';
import 'package:kanso/triage/models/repos/abs/triage_repository.dart';
import 'package:kanso/triage/state/triage/triage_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/mock_repositories.dart';

TriageChip chip(TriageField field, String value) => TriageChip(
  field: field,
  value: value,
  label: value,
  score: 1,
  because: const <TriageNote>[],
  ai: false,
  isNew: false,
);

TriageCard card(String id, List<TriageChip> chips, {String rev = 'r1'}) =>
    TriageCard(
      noteId: id,
      rev: rev,
      title: 'Note $id',
      snippet: '',
      updatedMs: 0,
      chips: chips,
      missing: TriageMissing.nothing,
    );

TriageFiled filed({int undo = 7, List<String> stale = const <String>[]}) =>
    TriageFiled(
      undo: undo,
      filed: 1,
      stale: stale,
      moved: 0,
      tags: 1,
      titles: 0,
      links: 0,
      sentence: 'Adds 1 tag.',
      message: 'Filed 1 note',
    );

/// A repository that answers from lists the test sets, and keeps what it
/// was asked, in order.
class FakeTriage implements TriageRepository {
  List<TriageCard> cardsNow = <TriageCard>[];
  final List<String> calls = <String>[];
  final List<List<TriageFiling>> filedWith = <List<TriageFiling>>[];
  final StreamController<List<String>> changed =
      StreamController<List<String>>.broadcast();

  /// Holds back the answer of [unreject] until completed.
  Completer<void>? holdUnreject;

  @override
  Future<Either<ApplicationError, TriageStatus>> status() async =>
      const Right<ApplicationError, TriageStatus>(
        TriageStatus(waiting: 1, relatedOn: true, aiOn: false),
      );

  @override
  Future<Either<ApplicationError, TriageSettings>> settings() async =>
      throw UnimplementedError();

  @override
  Future<Either<ApplicationError, TriageSettings>> setSettings(
    TriageSettings settings,
  ) async => throw UnimplementedError();

  @override
  Future<Either<ApplicationError, List<TriageCard>>> cards({
    required int limit,
  }) async => Right<ApplicationError, List<TriageCard>>(cardsNow);

  @override
  Future<Either<ApplicationError, List<TriageChip>>> refine(
    String noteId,
  ) async => const Right<ApplicationError, List<TriageChip>>(<TriageChip>[]);

  @override
  Future<Either<ApplicationError, Unit>> reject(
    String noteId,
    TriageField field,
    String value,
  ) async {
    calls.add('reject $value');
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, Unit>> unreject(
    String noteId,
    TriageField field,
    String value,
  ) async {
    calls.add('unreject:start $value');
    await holdUnreject?.future;
    calls.add('unreject:end $value');
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, TriageChip>> resolve(
    TriageField field,
    String text,
  ) async => Right<ApplicationError, TriageChip>(chip(field, text));

  @override
  Future<Either<ApplicationError, TriageFiled>> plan(
    List<TriageFiling> filings,
  ) async => Right<ApplicationError, TriageFiled>(filed(undo: 0));

  @override
  Future<Either<ApplicationError, TriageFiled>> file(
    List<TriageFiling> filings,
  ) async {
    filedWith.add(filings);
    return Right<ApplicationError, TriageFiled>(filed());
  }

  @override
  Future<Either<ApplicationError, Unit>> turnOnRelatedNotes() async =>
      const Right<ApplicationError, Unit>(unit);

  @override
  Stream<List<String>> changes() => changed.stream;
}

void main() {
  late FakeTriage triage;
  late MockUndoNoticeBloc undoNotice;
  late MockHapticsRepository haptics;
  late TriageBloc bloc;

  setUpAll(() {
    registerFallbackValue(const UndoNoticeEvent.dismissed());
  });

  setUp(() {
    triage = FakeTriage();
    undoNotice = MockUndoNoticeBloc();
    haptics = MockHapticsRepository();
    when(() => haptics.selection()).thenAnswer((_) async {});
    when(() => haptics.impact()).thenAnswer((_) async {});
    bloc = TriageBloc(triage, undoNotice, haptics);
    addTearDown(bloc.close);
    addTearDown(triage.changed.close);
  });

  Future<void> started() async {
    bloc.add(const TriageEvent.started());
    await bloc.stream.firstWhere((s) => s.status == LoadStatus.ready);
  }

  Future<void> settle() =>
      Future<void>.delayed(const Duration(milliseconds: 20));

  group('reading the cards', () {
    test('every card comes with its chips untouched', () async {
      triage.cardsNow = <TriageCard>[
        card('a', <TriageChip>[chip(TriageField.tag, 't1')]),
        card('b', <TriageChip>[]),
      ];
      await started();
      expect(bloc.state.cards.map((c) => c.noteId), <String>['a', 'b']);
      expect(bloc.state.cards.first.chips.single.mark, ChipMark.untouched);
      expect(bloc.state.info?.waiting, 1);
    });

    test(
      'a card read again with a new revision keeps accepted chips',
      () async {
        triage.cardsNow = <TriageCard>[
          card('a', <TriageChip>[
            chip(TriageField.tag, 't1'),
            chip(TriageField.tag, 't2'),
          ]),
        ];
        await started();
        bloc.add(const TriageEvent.chipAccepted('a', 0));
        await settle();
        // The note changed: other tags come back.
        triage.cardsNow = <TriageCard>[
          card('a', <TriageChip>[chip(TriageField.tag, 'other')], rev: 'r2'),
        ];
        bloc.add(const TriageEvent.reloaded());
        await bloc.stream.firstWhere((s) => s.cards.first.rev == 'r2');
        final values = bloc.state.cards.first.chips
            .map((c) => (c.chip.value, c.mark))
            .toList();
        expect(values, <(String, ChipMark)>[
          ('t1', ChipMark.accepted),
          ('other', ChipMark.untouched),
        ]);
      },
    );
  });

  group('accepting and rejecting', () {
    test(
      'a reject waits for the accept before it, on the same queue',
      () async {
        triage.cardsNow = <TriageCard>[
          card('a', <TriageChip>[chip(TriageField.tag, 't1')]),
        ];
        await started();
        // Reject it, then take the rejection back by accepting. The accept
        // waits on the repository; the reject that follows must not run
        // while it does.
        bloc.add(const TriageEvent.chipRejected('a', 0));
        await settle();
        triage.holdUnreject = Completer<void>();
        bloc
          ..add(const TriageEvent.chipAccepted('a', 0))
          ..add(const TriageEvent.chipRejected('a', 0));
        await settle();
        expect(triage.calls, <String>['reject t1', 'unreject:start t1']);
        triage.holdUnreject!.complete();
        await settle();
        expect(triage.calls, <String>[
          'reject t1',
          'unreject:start t1',
          'unreject:end t1',
          'reject t1',
        ]);
        expect(bloc.state.cards.single.chips.single.mark, ChipMark.rejected);
      },
    );
  });

  group('filing', () {
    test('File all accepted files the cards as they were planned', () async {
      triage.cardsNow = <TriageCard>[
        card('a', <TriageChip>[chip(TriageField.tag, 't1')]),
        card('b', <TriageChip>[chip(TriageField.tag, 't2')]),
      ];
      await started();
      bloc.add(const TriageEvent.cardAcceptedAll('a'));
      await settle();
      bloc.add(const TriageEvent.fileAllRequested());
      await bloc.stream.firstWhere((s) => s.plan != null);
      // After the question was shown, another card gets something accepted.
      bloc.add(const TriageEvent.cardAcceptedAll('b'));
      await settle();
      bloc.add(const TriageEvent.fileAllConfirmed());
      await settle();
      expect(triage.filedWith, hasLength(1));
      expect(triage.filedWith.single.map((f) => f.noteId), <String>['a']);
      expect(bloc.state.cards.map((c) => c.noteId), <String>['b']);
    });

    test('a no clears the plan, and a yes then files nothing', () async {
      triage.cardsNow = <TriageCard>[
        card('a', <TriageChip>[chip(TriageField.tag, 't1')]),
      ];
      await started();
      bloc.add(const TriageEvent.cardAcceptedAll('a'));
      await settle();
      bloc.add(const TriageEvent.fileAllRequested());
      await bloc.stream.firstWhere((s) => s.plan != null);
      bloc.add(const TriageEvent.planDismissed());
      await settle();
      bloc.add(const TriageEvent.fileAllConfirmed());
      await settle();
      expect(triage.filedWith, isEmpty);
    });

    test('filing one card takes it off the list and offers Undo', () async {
      triage.cardsNow = <TriageCard>[
        card('a', <TriageChip>[chip(TriageField.tag, 't1')]),
      ];
      await started();
      bloc.add(const TriageEvent.cardAcceptedAll('a'));
      await settle();
      bloc.add(const TriageEvent.cardFiled('a'));
      await settle();
      expect(bloc.state.cards, isEmpty);
      expect(triage.filedWith.single.single.accepted.single.value, 't1');
      verify(() => undoNotice.add(any())).called(1);
    });

    test('a card whose note changed is not filed', () async {
      triage.cardsNow = <TriageCard>[
        card('a', <TriageChip>[chip(TriageField.tag, 't1')]),
      ];
      await started();
      bloc.add(const TriageEvent.cardAcceptedAll('a'));
      await settle();
      triage.changed.add(<String>['a']);
      await bloc.stream.firstWhere((s) => s.cards.first.changed);
      bloc.add(const TriageEvent.cardFiled('a'));
      await settle();
      expect(triage.filedWith, isEmpty);
    });

    test('skipping hides the card for the visit', () async {
      triage.cardsNow = <TriageCard>[card('a', <TriageChip>[])];
      await started();
      bloc.add(const TriageEvent.cardSkipped('a'));
      await settle();
      expect(bloc.state.shown, isEmpty);
      expect(bloc.state.cards, hasLength(1));
    });
  });
}
