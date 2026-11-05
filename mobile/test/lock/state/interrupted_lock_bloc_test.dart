// Kanso — interrupted_lock_bloc_test.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/lock/state/interrupted_lock/interrupted_lock_bloc.dart';

import '../../helpers/fixture_session_repository.dart';

const _stopped = InterruptedLock(files: 5, sealed: 3);

Future<InterruptedLockState> _settled(
  InterruptedLockBloc bloc,
  bool Function(InterruptedLockState) done,
) => bloc.stream.firstWhere(done);

void main() {
  late FixtureSessionRepository session;
  late InterruptedLockBloc bloc;

  setUp(() {
    session = FixtureSessionRepository(unlocked: true)..stopped = _stopped;
    bloc = InterruptedLockBloc(session);
  });

  tearDown(() async {
    await bloc.close();
    await session.close();
  });

  group('InterruptedLockBloc finds a lock that stopped', () {
    test('nothing stopped: no bar', () async {
      session.stopped = null;
      bloc.add(const InterruptedLockEvent.checked('note:1'));
      final state = await _settled(
        bloc,
        (s) => s.stage == InterruptedStage.none,
      );
      expect(state.noteId, 'note:1');
    });

    test('a stopped lock says how far it got', () async {
      bloc.add(const InterruptedLockEvent.checked('note:1'));
      final state = await _settled(
        bloc,
        (s) => s.stage == InterruptedStage.stopped,
      );
      expect((state.sealed, state.files), (3, 5));
    });

    test('the notice about older locked notes is said once', () async {
      session
        ..filesNotice = true
        ..stopped = null;
      bloc.add(const InterruptedLockEvent.checked('note:1'));
      await _settled(bloc, (s) => s.filesNotice);

      bloc.add(const InterruptedLockEvent.noticeShown());
      await _settled(bloc, (s) => !s.filesNotice);
      // Asking again has nothing more to say.
      bloc.add(const InterruptedLockEvent.checked('note:1'));
      await Future<void>.delayed(const Duration(milliseconds: 20));
      expect(bloc.state.filesNotice, isFalse);
    });
  });

  group('InterruptedLockBloc finishes and undoes', () {
    test('finishing with the notes open finishes it', () async {
      bloc.add(const InterruptedLockEvent.checked('note:1'));
      await _settled(bloc, (s) => s.stage == InterruptedStage.stopped);

      bloc.add(const InterruptedLockEvent.finished());
      await _settled(bloc, (s) => s.stage == InterruptedStage.none);
      expect(session.calls, contains('finishLocking:note:1'));
      expect(session.stopped, isNull);
    });

    test('undoing with the notes open undoes it', () async {
      bloc.add(const InterruptedLockEvent.checked('note:1'));
      await _settled(bloc, (s) => s.stage == InterruptedStage.stopped);

      bloc.add(const InterruptedLockEvent.undone());
      await _settled(bloc, (s) => s.stage == InterruptedStage.none);
      expect(session.calls, contains('undoLocking:note:1'));
    });

    test('closed notes ask for the lock password, then finish', () async {
      session.unlocked = false;
      bloc.add(const InterruptedLockEvent.checked('note:1'));
      await _settled(bloc, (s) => s.stage == InterruptedStage.stopped);

      bloc.add(const InterruptedLockEvent.finished());
      final asked = await _settled(
        bloc,
        (s) => s.stage == InterruptedStage.needsPassword,
      );
      expect(asked.action, InterruptedAction.finish);

      bloc.add(const InterruptedLockEvent.finished(password: 'nope'));
      await _settled(bloc, (s) => s.stage == InterruptedStage.wrongPassword);

      bloc.add(InterruptedLockEvent.finished(password: session.password));
      await _settled(bloc, (s) => s.stage == InterruptedStage.none);
      expect(session.calls, contains('finishLocking:note:1'));
    });

    test('closed notes ask for the lock password, then undo', () async {
      session.unlocked = false;
      bloc.add(const InterruptedLockEvent.checked('note:1'));
      await _settled(bloc, (s) => s.stage == InterruptedStage.stopped);

      bloc.add(const InterruptedLockEvent.undone());
      final asked = await _settled(
        bloc,
        (s) => s.stage == InterruptedStage.needsPassword,
      );
      expect(asked.action, InterruptedAction.undo);

      bloc.add(InterruptedLockEvent.undone(password: session.password));
      await _settled(bloc, (s) => s.stage == InterruptedStage.none);
      expect(session.calls, contains('undoLocking:note:1'));
    });

    test('closing the password sheet leaves the bar as it was', () async {
      session.unlocked = false;
      bloc.add(const InterruptedLockEvent.checked('note:1'));
      await _settled(bloc, (s) => s.stage == InterruptedStage.stopped);
      bloc.add(const InterruptedLockEvent.finished());
      await _settled(bloc, (s) => s.stage == InterruptedStage.needsPassword);

      bloc.add(const InterruptedLockEvent.passwordDeclined());
      final back = await _settled(
        bloc,
        (s) => s.stage == InterruptedStage.stopped,
      );
      expect(back.action, isNull);
    });

    test('progress reaches the bar while finishing', () async {
      bloc.add(const InterruptedLockEvent.checked('note:1'));
      await _settled(bloc, (s) => s.stage == InterruptedStage.stopped);

      const progress = SealProgress(
        phase: SealPhase.sealing,
        done: 4,
        total: 5,
        bytesDone: 4,
        bytesTotal: 5,
      );
      bloc.add(const InterruptedLockEvent.progressed(progress));
      final seen = await _settled(bloc, (s) => s.progress != null);
      expect(seen.progress, progress);
    });
  });
}
