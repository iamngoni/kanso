// Kanso — dictation_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/dictation/models/data/dictation_capability.dart';
import 'package:kanso/dictation/models/data/dictation_failure.dart';
import 'package:kanso/dictation/models/data/dictation_permission.dart';
import 'package:kanso/dictation/models/data/dictation_signal.dart';
import 'package:kanso/dictation/models/data/dictation_status.dart';
import 'package:kanso/dictation/models/data/dictation_unavailable_reason.dart';
import 'package:kanso/dictation/state/dictation/dictation_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../helpers/fake_dictation_text_rules.dart';
import '../helpers/mock_dictation_repository.dart';

const _requesting = DictationState(status: DictationStatus.requesting);
const _listening = DictationState(status: DictationStatus.listening);

DictationState _heard(
  List<String> segments, {
  String partial = '',
  DictationStatus status = DictationStatus.listening,
}) => DictationState(
  status: status,
  segments: segments,
  partial: partial,
  finishedText: segments.join(' '),
  text: <String>[...segments, if (partial.isNotEmpty) partial].join(' '),
);

void main() {
  late SpokenRepository spoken;

  setUp(() => spoken = SpokenRepository());
  tearDown(() => spoken.close());

  Future<void> hear(DictationSignal signal) async {
    spoken.signals.add(signal);
    await pumpEventQueue();
  }

  group('checked', () {
    blocTest<DictationBloc, DictationState>(
      'says nothing when the phone can listen',
      build: () => DictationBloc(spoken.repository, PassThroughRules()),
      act: (bloc) => bloc.add(const DictationEvent.checked()),
      expect: () => <DictationState>[],
    );

    blocTest<DictationBloc, DictationState>(
      'says why a phone cannot',
      setUp: () => spoken = SpokenRepository(
        capability: const DictationCapability(
          available: false,
          onDevice: false,
          permission: DictationPermission.notDetermined,
          reason: DictationUnavailableReason.notOnDevice,
        ),
      ),
      build: () => DictationBloc(spoken.repository, PassThroughRules()),
      act: (bloc) => bloc.add(const DictationEvent.checked()),
      expect: () => <DictationState>[
        const DictationState(
          status: DictationStatus.unavailable,
          reason: DictationUnavailableReason.notOnDevice,
        ),
      ],
    );

    blocTest<DictationBloc, DictationState>(
      'says when the microphone was refused',
      setUp: () => spoken = SpokenRepository(
        capability: const DictationCapability(
          available: true,
          onDevice: true,
          permission: DictationPermission.denied,
        ),
      ),
      build: () => DictationBloc(spoken.repository, PassThroughRules()),
      act: (bloc) => bloc.add(const DictationEvent.checked()),
      expect: () => <DictationState>[
        const DictationState(status: DictationStatus.denied),
      ],
    );

    blocTest<DictationBloc, DictationState>(
      'says when the phone bars listening',
      setUp: () => spoken = SpokenRepository(
        capability: const DictationCapability(
          available: true,
          onDevice: true,
          permission: DictationPermission.restricted,
        ),
      ),
      build: () => DictationBloc(spoken.repository, PassThroughRules()),
      act: (bloc) => bloc.add(const DictationEvent.checked()),
      expect: () => <DictationState>[
        const DictationState(status: DictationStatus.denied, restricted: true),
      ],
    );
  });

  group('started', () {
    blocTest<DictationBloc, DictationState>(
      'listens, shows the phrase being spoken and keeps the finished ones',
      build: () => DictationBloc(spoken.repository, PassThroughRules()),
      act: (bloc) async {
        bloc.add(const DictationEvent.started());
        await pumpEventQueue();
        await hear(const DictationSignal.listening());
        await hear(const DictationSignal.partial('hello'));
        await hear(const DictationSignal.partial('hello there'));
        await hear(const DictationSignal.finalText('Hello there.'));
        await hear(const DictationSignal.partial('and'));
        await hear(const DictationSignal.level(0.5));
      },
      expect: () => <DictationState>[
        _requesting,
        _listening,
        _heard(const <String>[], partial: 'hello'),
        _heard(const <String>[], partial: 'hello there'),
        _heard(const <String>['Hello there.']),
        _heard(const <String>['Hello there.'], partial: 'and'),
        _heard(const <String>['Hello there.'], partial: 'and').copyWith(
          level: 0.5,
        ),
      ],
      verify: (_) => verify(
        () => spoken.repository.start(localeTag: any(named: 'localeTag')),
      ).called(1),
    );

    blocTest<DictationBloc, DictationState>(
      'asks for the permission that is not asked yet, then listens',
      setUp: () => spoken = SpokenRepository(
        capability: const DictationCapability(
          available: true,
          onDevice: true,
          permission: DictationPermission.notDetermined,
        ),
      ),
      build: () => DictationBloc(spoken.repository, PassThroughRules()),
      act: (bloc) => bloc.add(const DictationEvent.started()),
      expect: () => <DictationState>[_requesting],
      verify: (_) {
        verify(spoken.repository.requestPermission).called(1);
        verify(
          () => spoken.repository.start(localeTag: any(named: 'localeTag')),
        ).called(1);
      },
    );

    blocTest<DictationBloc, DictationState>(
      'does not listen when the permission is refused',
      setUp: () => spoken = SpokenRepository(
        capability: const DictationCapability(
          available: true,
          onDevice: true,
          permission: DictationPermission.notDetermined,
        ),
        asked: DictationPermission.denied,
      ),
      build: () => DictationBloc(spoken.repository, PassThroughRules()),
      act: (bloc) => bloc.add(const DictationEvent.started()),
      expect: () => <DictationState>[
        _requesting,
        const DictationState(status: DictationStatus.denied),
      ],
      verify: (_) => verifyNever(
        () => spoken.repository.start(localeTag: any(named: 'localeTag')),
      ),
    );

    blocTest<DictationBloc, DictationState>(
      'does not listen where the phone cannot turn speech into text',
      setUp: () => spoken = SpokenRepository(
        capability: const DictationCapability(
          available: false,
          onDevice: false,
          permission: DictationPermission.granted,
          reason: DictationUnavailableReason.unsupportedLanguage,
        ),
      ),
      build: () => DictationBloc(spoken.repository, PassThroughRules()),
      act: (bloc) => bloc.add(const DictationEvent.started()),
      expect: () => <DictationState>[
        _requesting,
        const DictationState(
          status: DictationStatus.unavailable,
          reason: DictationUnavailableReason.unsupportedLanguage,
        ),
      ],
    );

    blocTest<DictationBloc, DictationState>(
      'is not started twice',
      build: () => DictationBloc(spoken.repository, PassThroughRules()),
      act: (bloc) async {
        bloc.add(const DictationEvent.started());
        await pumpEventQueue();
        bloc.add(const DictationEvent.started());
        await pumpEventQueue();
      },
      expect: () => <DictationState>[_requesting],
      verify: (_) => verify(
        () => spoken.repository.start(localeTag: any(named: 'localeTag')),
      ).called(1),
    );
  });

  group('the end of a session', () {
    blocTest<DictationBloc, DictationState>(
      'keeps the words not yet finished when it ends',
      build: () => DictationBloc(spoken.repository, PassThroughRules()),
      act: (bloc) async {
        bloc.add(const DictationEvent.started());
        await pumpEventQueue();
        await hear(const DictationSignal.partial('buy milk'));
        bloc.add(const DictationEvent.stopped());
        await pumpEventQueue();
        await hear(const DictationSignal.ended());
      },
      expect: () => <DictationState>[
        _requesting,
        _heard(const <String>[], partial: 'buy milk'),
        const DictationState(
          segments: <String>['buy milk'],
          finishedText: 'buy milk',
          text: 'buy milk',
        ),
      ],
      verify: (_) => verify(spoken.repository.stop).called(1),
    );

    blocTest<DictationBloc, DictationState>(
      'shows a failure and keeps what was heard before it',
      build: () => DictationBloc(spoken.repository, PassThroughRules()),
      act: (bloc) async {
        bloc.add(const DictationEvent.started());
        await pumpEventQueue();
        await hear(const DictationSignal.finalText('One.'));
        await hear(const DictationSignal.failed(DictationFailure.interrupted));
        await hear(const DictationSignal.ended());
      },
      expect: () => <DictationState>[
        _requesting,
        _heard(const <String>['One.']),
        _heard(const <String>['One.']).copyWith(
          status: DictationStatus.error,
          failure: DictationFailure.interrupted,
        ),
      ],
    );

    blocTest<DictationBloc, DictationState>(
      'turns a refusal the platform reports into the denied state',
      build: () => DictationBloc(spoken.repository, PassThroughRules()),
      act: (bloc) async {
        bloc.add(const DictationEvent.started());
        await pumpEventQueue();
        await hear(
          const DictationSignal.failed(DictationFailure.permissionDenied),
        );
      },
      expect: () => <DictationState>[
        _requesting,
        const DictationState(status: DictationStatus.denied),
      ],
    );

    blocTest<DictationBloc, DictationState>(
      'ignores the signals of a session another screen started',
      build: () => DictationBloc(spoken.repository, PassThroughRules()),
      act: (bloc) async {
        await hear(const DictationSignal.partial('not mine'));
        await hear(const DictationSignal.ended());
      },
      expect: () => <DictationState>[],
    );
  });

  group('cancelled', () {
    test('cancelling while availability is pending prevents start', () async {
      final availability =
          Completer<Either<ApplicationError, DictationCapability>>();
      when(
        () => spoken.repository.availability(
          localeTag: any(named: 'localeTag'),
        ),
      ).thenAnswer((_) => availability.future);
      final bloc = DictationBloc(spoken.repository, PassThroughRules())
        ..add(const DictationEvent.started());
      await bloc.stream.firstWhere((state) => state.active);

      bloc.add(const DictationEvent.cancelled());
      await bloc.stream.firstWhere((state) => !state.active);
      availability.complete(
        const Right<ApplicationError, DictationCapability>(
          DictationCapability(
            available: true,
            onDevice: true,
            permission: DictationPermission.granted,
          ),
        ),
      );
      await pumpEventQueue();

      verify(spoken.repository.signals).called(1);
      verify(
        () => spoken.repository.availability(
          localeTag: any(named: 'localeTag'),
        ),
      ).called(1);
      verify(spoken.repository.cancel).called(1);
      verifyNever(
        () => spoken.repository.start(localeTag: any(named: 'localeTag')),
      );
      await bloc.close();
    });

    blocTest<DictationBloc, DictationState>(
      'drops what was heard and what comes after',
      build: () => DictationBloc(spoken.repository, PassThroughRules()),
      act: (bloc) async {
        bloc.add(const DictationEvent.started());
        await pumpEventQueue();
        await hear(const DictationSignal.finalText('Lost.'));
        bloc.add(const DictationEvent.cancelled());
        await pumpEventQueue();
        await hear(const DictationSignal.finalText('Late.'));
        await hear(const DictationSignal.ended());
      },
      expect: () => <DictationState>[
        _requesting,
        _heard(const <String>['Lost.']),
        const DictationState(),
      ],
      verify: (_) => verify(spoken.repository.cancel).called(1),
    );
  });

  group('finishRequested', () {
    blocTest<DictationBloc, DictationState>(
      'stops, and says it is finished once the recognizer has ended',
      build: () => DictationBloc(spoken.repository, PassThroughRules()),
      act: (bloc) async {
        bloc.add(const DictationEvent.started());
        await pumpEventQueue();
        await hear(const DictationSignal.partial('last words'));
        bloc.add(const DictationEvent.finishRequested());
        await pumpEventQueue();
        await hear(const DictationSignal.finalText('Last words.'));
        await hear(const DictationSignal.ended());
      },
      expect: () => <DictationState>[
        _requesting,
        _heard(const <String>[], partial: 'last words'),
        _heard(const <String>['Last words.']),
        const DictationState(
          segments: <String>['Last words.'],
          finishedText: 'Last words.',
          text: 'Last words.',
          finished: true,
        ),
      ],
      verify: (_) => verify(spoken.repository.stop).called(1),
    );

    blocTest<DictationBloc, DictationState>(
      'is finished at once when nothing listens',
      build: () => DictationBloc(spoken.repository, PassThroughRules()),
      act: (bloc) => bloc.add(const DictationEvent.finishRequested()),
      expect: () => <DictationState>[const DictationState(finished: true)],
      verify: (_) => verifyNever(spoken.repository.stop),
    );
  });

  test('the transcript is the finished phrases and the one being spoken', () {
    expect(
      _heard(const <String>['One.', 'Two.'], partial: 'three').transcript,
      'One. Two. three',
    );
    expect(const DictationState().transcript, isEmpty);
  });

  blocTest<DictationBloc, DictationState>(
    'opens the phone settings when asked',
    build: () => DictationBloc(spoken.repository, PassThroughRules()),
    act: (bloc) => bloc.add(const DictationEvent.settingsRequested()),
    expect: () => <DictationState>[],
    verify: (_) => verify(spoken.repository.openSettings).called(1),
  );
}
