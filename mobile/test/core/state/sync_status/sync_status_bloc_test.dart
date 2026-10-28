// Kanso — sync_status_bloc_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/state/sync_status/sync_status_bloc.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/mock_repositories.dart';

const _signedOut = SyncStatus(
  phase: SyncPhase.signedOut,
  conflicts: 0,
  skippedCount: 0,
  account: SyncAccount.signedOut(),
);
const _idle = SyncStatus(
  phase: SyncPhase.idle,
  username: 'ana',
  server: 'https://example.invalid',
  conflicts: 0,
  skippedCount: 0,
  account: SyncAccount.signedIn(),
);
const _syncing = SyncStatus(
  phase: SyncPhase.syncing,
  username: 'ana',
  conflicts: 0,
  skippedCount: 0,
  account: SyncAccount.signedIn(),
);
const _report = SyncReportRow(
  pulled: 1,
  pushed: 0,
  conflicts: 0,
  pending: 0,
  queued: false,
);
const _offline = ApplicationError.network(message: 'Offline.');

void main() {
  late MockSyncRepository sync;
  late StreamController<SyncStatus> statuses;

  setUp(() {
    sync = MockSyncRepository();
    statuses = StreamController<SyncStatus>.broadcast();
    addTearDown(statuses.close);
    when(() => sync.statusStream()).thenAnswer((_) => statuses.stream);
    when(() => sync.syncNow()).thenAnswer(
      (_) async => const Right<ApplicationError, SyncReportRow>(_report),
    );
  });

  SyncStatusBloc build() => SyncStatusBloc(sync);

  group('SyncStatusBloc', () {
    test('starts with no status, not signed in and not busy', () {
      final state = build().state;
      expect(state.status, isNull);
      expect(state.signedIn, isFalse);
      expect(state.busy, isFalse);
    });

    blocTest<SyncStatusBloc, SyncStatusState>(
      'follows the status as it changes',
      build: build,
      act: (bloc) async {
        bloc.add(const SyncStatusEvent.started());
        await Future<void>.delayed(const Duration(milliseconds: 5));
        statuses
          ..add(_signedOut)
          ..add(_idle)
          ..add(_syncing);
      },
      wait: const Duration(milliseconds: 20),
      expect: () => const <SyncStatusState>[
        SyncStatusState(status: _signedOut),
        SyncStatusState(status: _idle),
        SyncStatusState(status: _syncing),
      ],
      verify: (bloc) {
        expect(bloc.state.signedIn, isTrue);
        expect(bloc.state.busy, isTrue);
      },
    );

    test('signed out is not signed in, and not busy', () {
      const state = SyncStatusState(status: _signedOut);
      expect(state.signedIn, isFalse);
      expect(state.busy, isFalse);
    });

    blocTest<SyncStatusBloc, SyncStatusState>(
      'syncRequested syncs and says so while it runs',
      build: build,
      seed: () => const SyncStatusState(status: _idle),
      act: (bloc) => bloc.add(const SyncStatusEvent.syncRequested()),
      expect: () => const <SyncStatusState>[
        SyncStatusState(status: _idle, syncing: true),
        SyncStatusState(status: _idle),
      ],
      verify: (_) => verify(() => sync.syncNow()).called(1),
    );

    blocTest<SyncStatusBloc, SyncStatusState>(
      'a failed sync keeps the status and says why',
      setUp: () => when(() => sync.syncNow()).thenAnswer(
        (_) async => const Left<ApplicationError, SyncReportRow>(_offline),
      ),
      build: build,
      seed: () => const SyncStatusState(status: _idle),
      act: (bloc) => bloc.add(const SyncStatusEvent.syncRequested()),
      expect: () => const <SyncStatusState>[
        SyncStatusState(status: _idle, syncing: true),
        SyncStatusState(status: _idle, error: _offline),
      ],
    );

    blocTest<SyncStatusBloc, SyncStatusState>(
      'a sync that works clears the error of the last',
      build: build,
      seed: () => const SyncStatusState(status: _idle, error: _offline),
      act: (bloc) => bloc.add(const SyncStatusEvent.syncRequested()),
      verify: (bloc) => expect(bloc.state.error, isNull),
    );

    blocTest<SyncStatusBloc, SyncStatusState>(
      'asking again while a sync runs does not start another',
      setUp: () {
        final done = Completer<Either<ApplicationError, SyncReportRow>>();
        when(() => sync.syncNow()).thenAnswer((_) => done.future);
        Timer(
          const Duration(milliseconds: 30),
          () => done.complete(
            const Right<ApplicationError, SyncReportRow>(_report),
          ),
        );
      },
      build: build,
      act: (bloc) async {
        bloc.add(const SyncStatusEvent.syncRequested());
        await Future<void>.delayed(const Duration(milliseconds: 5));
        bloc.add(const SyncStatusEvent.syncRequested());
      },
      wait: const Duration(milliseconds: 60),
      verify: (_) => verify(() => sync.syncNow()).called(1),
    );

    test(
      'tells the library when the app is in front and when it is not',
      () async {
        when(
          () => sync.setForeground(foreground: any(named: 'foreground')),
        ).thenAnswer(
          (_) async => const Right<ApplicationError, Unit>(unit),
        );
        final bloc = build()
          ..add(const SyncStatusEvent.foregroundChanged(foreground: true))
          ..add(const SyncStatusEvent.foregroundChanged(foreground: false))
          ..add(const SyncStatusEvent.foregroundChanged(foreground: true));
        await pumpEventQueue();
        verifyInOrder(<void Function()>[
          () => sync.setForeground(foreground: true),
          () => sync.setForeground(foreground: false),
          () => sync.setForeground(foreground: true),
        ]);
        await bloc.close();
      },
    );

    test('a failure to say so changes nothing the person sees', () async {
      when(
        () => sync.setForeground(foreground: any(named: 'foreground')),
      ).thenAnswer(
        (_) async => const Left<ApplicationError, Unit>(_offline),
      );
      final bloc = build()
        ..add(const SyncStatusEvent.foregroundChanged(foreground: true));
      await pumpEventQueue();
      expect(bloc.state.error, isNull);
      expect(bloc.state.busy, isFalse);
      await bloc.close();
    });

    test('lets go of the status stream when it is closed', () async {
      final bloc = build()..add(const SyncStatusEvent.started());
      await Future<void>.delayed(const Duration(milliseconds: 10));
      expect(statuses.hasListener, isTrue);
      await bloc.close();
      expect(statuses.hasListener, isFalse);
    });
  });
}
