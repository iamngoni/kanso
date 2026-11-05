// Kanso — files_migration_bloc_test.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/lock/state/files_migration/files_migration_bloc.dart';

import '../../helpers/fixture_session_repository.dart';

FilesMigration _waiting({
  int notes = 3,
  int stopped = 0,
  bool postponed = false,
  bool counted = false,
}) => FilesMigration(
  notes: notes,
  stopped: stopped,
  postponed: postponed,
  counted: counted,
  files: counted ? 12 : 0,
  bytes: counted ? 84000000 : 0,
  shared: counted ? 3 : 0,
);

const MigrationOutcome _sealedAll = MigrationOutcome(
  notes: 3,
  sealed: 9,
  copied: 3,
  unopened: 0,
  failed: 0,
  stopped: false,
);

Future<FilesMigrationState> _settled(
  FilesMigrationBloc bloc,
  bool Function(FilesMigrationState) done,
) => bloc.stream.firstWhere(done);

void main() {
  late FixtureSessionRepository session;
  late FilesMigrationBloc bloc;

  setUp(() {
    session = FixtureSessionRepository()..migration = _waiting();
    bloc = FilesMigrationBloc(session);
  });

  tearDown(() async {
    await bloc.close();
    await session.close();
  });

  group('FilesMigrationBloc at the first launch', () {
    test('asks when notes wait and Later was not chosen', () async {
      bloc.add(const FilesMigrationEvent.checked(launch: true));
      final state = await _settled(bloc, (s) => s.stage == MigrationStage.ask);
      expect(state.info?.notes, 3);
    });

    test('reminds quietly after Later', () async {
      session.migration = _waiting(postponed: true);
      bloc.add(const FilesMigrationEvent.checked(launch: true));
      await _settled(bloc, (s) => s.stage == MigrationStage.remind);
    });

    test('says nothing when every attachment is sealed', () async {
      session.migration = _waiting(notes: 0);
      bloc.add(const FilesMigrationEvent.checked(launch: true));
      await _settled(bloc, (s) => s.stage == MigrationStage.idle);
    });

    test('Later is remembered at launch and not from Settings', () async {
      bloc.add(const FilesMigrationEvent.checked(launch: true));
      await _settled(bloc, (s) => s.stage == MigrationStage.ask);
      bloc.add(const FilesMigrationEvent.postponed());
      await _settled(bloc, (s) => s.stage == MigrationStage.idle);
      expect(session.calls, contains('postponeFilesMigration'));

      session.calls.clear();
      bloc.add(const FilesMigrationEvent.checked());
      await Future<void>.delayed(const Duration(milliseconds: 10));
      bloc.add(const FilesMigrationEvent.opened());
      await _settled(bloc, (s) => s.stage == MigrationStage.ask);
      bloc.add(const FilesMigrationEvent.postponed());
      await _settled(bloc, (s) => s.stage == MigrationStage.idle);
      expect(session.calls, isNot(contains('postponeFilesMigration')));
    });
  });

  group('FilesMigrationBloc seals', () {
    Future<void> asked() async {
      bloc.add(const FilesMigrationEvent.checked(launch: true));
      await _settled(bloc, (s) => s.stage == MigrationStage.ask);
    }

    test('closed notes ask for the lock password first', () async {
      session
        ..unlocked = false
        ..migrated = _sealedAll;
      await asked();
      bloc.add(const FilesMigrationEvent.sealRequested());
      await _settled(bloc, (s) => s.stage == MigrationStage.password);

      bloc.add(const FilesMigrationEvent.sealed(password: 'nope'));
      await _settled(bloc, (s) => s.stage == MigrationStage.wrongPassword);

      bloc.add(FilesMigrationEvent.sealed(password: session.password));
      final done = await _settled(bloc, (s) => s.stage == MigrationStage.done);
      expect(done.outcome, _sealedAll);
      expect(session.calls, contains('sealLockedAttachments'));
    });

    test('open notes seal at once, counted', () async {
      session
        ..unlocked = true
        ..migration = _waiting(counted: true)
        ..migrated = _sealedAll;
      await asked();
      bloc.add(const FilesMigrationEvent.sealRequested());
      final done = await _settled(bloc, (s) => s.stage == MigrationStage.done);
      expect(done.outcome?.sealed, 9);
      // What waits is read again: nothing now.
      final after = await _settled(bloc, (s) => (s.info?.notes ?? 1) == 0);
      expect(after.stage, MigrationStage.done);
    });

    test('progress reaches the sheet while it seals', () async {
      session
        ..unlocked = true
        ..migration = _waiting(counted: true)
        ..migrationGate = Completer<void>();
      await asked();
      bloc.add(const FilesMigrationEvent.sealRequested());
      await _settled(bloc, (s) => s.stage == MigrationStage.working);
      session.progress.add(
        const SealProgress(
          phase: SealPhase.sealing,
          done: 5,
          total: 12,
          bytesDone: 31000000,
          bytesTotal: 84000000,
        ),
      );
      final seen = await _settled(bloc, (s) => s.progress != null);
      expect((seen.progress?.done, seen.progress?.total), (5, 12));
      session.migrationGate?.complete();
      await _settled(bloc, (s) => s.stage == MigrationStage.done);
    });

    test('Cancel stops the pass between two files', () async {
      session
        ..unlocked = true
        ..migration = _waiting(counted: true)
        ..migrationGate = Completer<void>();
      session.gate = session.migrationGate;
      await asked();
      bloc.add(const FilesMigrationEvent.sealRequested());
      await _settled(bloc, (s) => s.stage == MigrationStage.working);
      bloc.add(const FilesMigrationEvent.cancelled());
      final stopping = await _settled(bloc, (s) => s.cancelling);
      expect(stopping.stage, MigrationStage.working);
      expect(session.calls, contains('cancelLock'));
    });
  });

  group('FilesMigrationBloc after a pass that stopped', () {
    test('finishes, or undoes with the lock password', () async {
      session
        ..unlocked = false
        ..migration = _waiting(stopped: 1, notes: 1);
      bloc.add(const FilesMigrationEvent.checked(launch: true));
      await _settled(bloc, (s) => s.stage == MigrationStage.ask);
      expect(bloc.state.info?.stopped, 1);

      bloc.add(const FilesMigrationEvent.undoRequested());
      final asked = await _settled(
        bloc,
        (s) => s.stage == MigrationStage.password,
      );
      expect(asked.action, MigrationAction.undo);

      bloc.add(FilesMigrationEvent.undone(password: session.password));
      final done = await _settled(bloc, (s) => s.stage == MigrationStage.done);
      expect(done.undone, 2);
      expect(session.calls, contains('undoFilesMigration'));
    });
  });
}
