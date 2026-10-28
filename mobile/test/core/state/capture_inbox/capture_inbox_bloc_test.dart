// Kanso — capture_inbox_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/state/capture_inbox/capture_inbox_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/mock_repositories.dart';

Either<ApplicationError, DropsSaved> _saved(
  List<String> notes, {
  List<String> problems = const <String>[],
  int kept = 0,
}) => Right<ApplicationError, DropsSaved>(
  DropsSaved(notes: notes, problems: problems, kept: kept),
);

void main() {
  late MockCaptureRepository capture;

  setUp(() {
    capture = MockCaptureRepository();
  });

  CaptureInboxBloc build() => CaptureInboxBloc(capture);

  group('CaptureInboxBloc', () {
    test('starts idle with nothing saved', () {
      final state = build().state;
      expect(state.saving, isFalse);
      expect(state.savedCount, 0);
      expect(state.problems, isEmpty);
      expect(state.error, isNull);
    });

    blocTest<CaptureInboxBloc, CaptureInboxState>(
      'saves the waiting items and reports the notes made',
      setUp: () => when(
        () => capture.saveDrops(),
      ).thenAnswer((_) async => _saved(<String>['note:cap-a', 'note:cap-b'])),
      build: build,
      act: (bloc) => bloc.add(const CaptureInboxEvent.saveRequested()),
      expect: () => const <CaptureInboxState>[
        CaptureInboxState(saving: true),
        CaptureInboxState(
          saved: <String>['note:cap-a', 'note:cap-b'],
          savedCount: 2,
        ),
      ],
      verify: (_) => verify(() => capture.saveDrops()).called(1),
    );

    blocTest<CaptureInboxBloc, CaptureInboxState>(
      'keeps what was left out, and what was kept for later, in the state',
      setUp: () => when(() => capture.saveDrops()).thenAnswer(
        (_) async => _saved(
          <String>[],
          problems: <String>['a.zip: Only images and PDFs.'],
          kept: 1,
        ),
      ),
      build: build,
      act: (bloc) => bloc.add(const CaptureInboxEvent.saveRequested()),
      expect: () => const <CaptureInboxState>[
        CaptureInboxState(saving: true),
        CaptureInboxState(
          kept: 1,
          problems: <String>['a.zip: Only images and PDFs.'],
        ),
      ],
    );

    blocTest<CaptureInboxBloc, CaptureInboxState>(
      'problems add up until they are dismissed',
      setUp: () {
        final answers = <Either<ApplicationError, DropsSaved>>[
          _saved(<String>[], problems: <String>['one']),
          _saved(<String>['note:cap-c'], problems: <String>['two']),
        ];
        when(
          () => capture.saveDrops(),
        ).thenAnswer((_) async => answers.removeAt(0));
      },
      build: build,
      act: (bloc) async {
        bloc.add(const CaptureInboxEvent.saveRequested());
        await Future<void>.delayed(Duration.zero);
        bloc.add(const CaptureInboxEvent.saveRequested());
        await Future<void>.delayed(Duration.zero);
        bloc.add(const CaptureInboxEvent.problemsDismissed());
      },
      verify: (bloc) {
        expect(bloc.state.problems, isEmpty);
        expect(bloc.state.savedCount, 1);
        expect(bloc.state.saved, <String>['note:cap-c']);
      },
      expect: () => const <CaptureInboxState>[
        CaptureInboxState(saving: true),
        CaptureInboxState(problems: <String>['one']),
        CaptureInboxState(saving: true, problems: <String>['one']),
        CaptureInboxState(
          saved: <String>['note:cap-c'],
          savedCount: 1,
          problems: <String>['one', 'two'],
        ),
        CaptureInboxState(
          saved: <String>['note:cap-c'],
          savedCount: 1,
        ),
      ],
    );

    blocTest<CaptureInboxBloc, CaptureInboxState>(
      'a failed save is an error, and a later one clears it',
      setUp: () {
        final answers = <Either<ApplicationError, DropsSaved>>[
          const Left<ApplicationError, DropsSaved>(
            ApplicationError.storage(message: 'Disk full.'),
          ),
          _saved(<String>[]),
        ];
        when(
          () => capture.saveDrops(),
        ).thenAnswer((_) async => answers.removeAt(0));
      },
      build: build,
      act: (bloc) async {
        bloc.add(const CaptureInboxEvent.saveRequested());
        await Future<void>.delayed(Duration.zero);
        bloc.add(const CaptureInboxEvent.saveRequested());
      },
      expect: () => const <CaptureInboxState>[
        CaptureInboxState(saving: true),
        CaptureInboxState(
          error: ApplicationError.storage(message: 'Disk full.'),
        ),
        CaptureInboxState(
          saving: true,
          error: ApplicationError.storage(message: 'Disk full.'),
        ),
        CaptureInboxState(),
      ],
    );

    blocTest<CaptureInboxBloc, CaptureInboxState>(
      'a save asked for while one runs is dropped, so two never overlap',
      setUp: () {
        final gate = Completer<Either<ApplicationError, DropsSaved>>();
        when(() => capture.saveDrops()).thenAnswer((_) => gate.future);
        addTearDown(() {
          if (!gate.isCompleted) {
            gate.complete(_saved(<String>[]));
          }
        });
      },
      build: build,
      act: (bloc) async {
        bloc
          ..add(const CaptureInboxEvent.saveRequested())
          ..add(const CaptureInboxEvent.saveRequested())
          ..add(const CaptureInboxEvent.saveRequested());
        await Future<void>.delayed(const Duration(milliseconds: 5));
      },
      expect: () => const <CaptureInboxState>[CaptureInboxState(saving: true)],
      verify: (_) => verify(() => capture.saveDrops()).called(1),
    );
  });
}
