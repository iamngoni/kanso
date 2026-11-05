// Kanso — interrupted_lock_test.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/views/widgets/kanso_feedback.dart';
import 'package:kanso/lock/models/data/lock_target.dart';
import 'package:kanso/lock/state/lock_status/lock_status_bloc.dart';
import 'package:kanso/lock/views/widgets/interrupted_lock_bar.dart';
import 'package:kanso/lock/views/widgets/interrupted_lock_host.dart';
import 'package:kanso/lock/views/widgets/lock_item_sheet.dart';
import 'package:kanso/lock/views/widgets/lock_progress_sheet.dart';
import 'package:kanso/lock/views/widgets/remove_lock_sheet.dart';

import '../../../helpers/fixture_device_lock_repository.dart';
import '../../../helpers/fixture_session_repository.dart';
import '../../../helpers/kanso_test_app.dart';
import '../../../helpers/pump.dart';
import '../../../helpers/register_lock_blocs.dart';

const InterruptedLock _stopped = InterruptedLock(files: 5, sealed: 3);

Future<FixtureSessionRepository> _pumpHost(
  WidgetTester tester, {
  required bool unlocked,
  InterruptedLock? stopped = _stopped,
  bool filesNotice = false,
}) async {
  final session = FixtureSessionRepository(unlocked: unlocked)
    ..stopped = stopped
    ..filesNotice = filesNotice;
  addTearDown(session.close);
  registerLockBlocs(session, FixtureDeviceLockRepository());
  await pumpKanso(
    tester,
    const ScaffoldMessenger(
      child: Scaffold(
        body: InterruptedLockHost(noteId: 'note:1', revision: 'r1', gutter: 16),
      ),
    ),
  );
  return session;
}

/// What the sheets said through the app's feedback pill. The pill's host
/// wraps the app's navigator in the app; a test keeps its controller above
/// the test app instead and reads what was said.
late KansoFeedbackController _said;

/// What was said, taken: its timer goes with it.
String? _heard() {
  final text = _said.current?.text;
  _said.hide();
  return text;
}

Future<void> _pumpSheetPage(WidgetTester tester, Widget child) async {
  _said = KansoFeedbackController();
  addTearDown(_said.dispose);
  await tester.pumpWidget(
    KansoFeedbackScope(
      controller: _said,
      child: KansoTestApp(home: Center(child: child)),
    ),
  );
  await tester.pumpAndSettle();
}

void main() {
  group('the bar above a note whose lock stopped', () {
    testWidgets('says how far it got and offers both ways out', (
      tester,
    ) async {
      await _pumpHost(tester, unlocked: true);
      expect(find.byType(InterruptedLockBar), findsOneWidget);
      expect(
        find.textContaining('3 of its 5 pictures and files are sealed'),
        findsOneWidget,
      );
      expect(find.text('Finish locking'), findsOneWidget);
      expect(find.text('Undo locking'), findsOneWidget);
    });

    testWidgets('draws nothing when no lock stopped', (tester) async {
      await _pumpHost(tester, unlocked: true, stopped: null);
      expect(find.text('Finish locking'), findsNothing);
    });

    testWidgets('Finish locking finishes it and the bar goes', (tester) async {
      final session = await _pumpHost(tester, unlocked: true);
      await tester.tap(find.text('Finish locking'));
      await tester.pumpAndSettle();
      expect(session.calls, contains('finishLocking:note:1'));
      expect(find.text('Finish locking'), findsNothing);
    });

    testWidgets('Undo locking undoes it and the bar goes', (tester) async {
      final session = await _pumpHost(tester, unlocked: true);
      await tester.tap(find.text('Undo locking'));
      await tester.pumpAndSettle();
      expect(session.calls, contains('undoLocking:note:1'));
      expect(find.text('Undo locking'), findsNothing);
    });

    testWidgets('closed notes ask for the lock password first', (
      tester,
    ) async {
      final session = await _pumpHost(tester, unlocked: false);
      await tester.tap(find.text('Finish locking'));
      await tester.pumpAndSettle();
      expect(find.text('Enter the lock password'), findsOneWidget);

      await tester.enterText(find.byType(EditableText).last, 'not it');
      await tester.tap(find.text('Continue'));
      await tester.pumpAndSettle();
      expect(find.text("That isn't the lock password."), findsOneWidget);

      await tester.enterText(find.byType(EditableText).last, session.password);
      await tester.tap(find.text('Continue'));
      await tester.pumpAndSettle();
      expect(session.calls, contains('finishLocking:note:1'));
      expect(find.text('Enter the lock password'), findsNothing);
      expect(find.text('Finish locking'), findsNothing);
    });

    testWidgets('closing the password sheet keeps the bar', (tester) async {
      await _pumpHost(tester, unlocked: false);
      await tester.tap(find.text('Undo locking'));
      await tester.pumpAndSettle();
      expect(find.text('Enter the lock password'), findsOneWidget);

      await tester.tap(find.text('Cancel'));
      await tester.pumpAndSettle();
      expect(find.text('Enter the lock password'), findsNothing);
      expect(find.text('Undo locking'), findsOneWidget);
    });

    testWidgets('says once that older locked notes seal their files', (
      tester,
    ) async {
      await _pumpHost(
        tester,
        unlocked: true,
        stopped: null,
        filesNotice: true,
      );
      expect(
        find.textContaining('Locked notes now seal their pictures and files'),
        findsOneWidget,
      );
    });
  });

  group('the progress sheet', () {
    Future<(FixtureSessionRepository, Completer<void>)> pumpLock(
      WidgetTester tester, {
      required LockSummary summary,
    }) async {
      final session = FixtureSessionRepository(unlocked: true)
        ..summary = summary;
      final gate = Completer<void>();
      session.gate = gate;
      addTearDown(session.close);
      registerLockBlocs(session, FixtureDeviceLockRepository());
      final status = LockStatusBloc(session, FixtureDeviceLockRepository());
      addTearDown(status.close);
      await _pumpSheetPage(
        tester,
        BlocProvider<LockStatusBloc>.value(
          value: status,
          child: Builder(
            builder: (context) => TextButton(
              onPressed: () => unawaited(
                LockItemSheet.show(
                  context,
                  const LockTarget.note('note:1'),
                  status: status,
                ),
              ),
              child: const Text('open'),
            ),
          ),
        ),
      );
      await tester.tap(find.text('open'));
      await tester.pumpAndSettle();
      return (session, gate);
    }

    testWidgets('a long lock counts its files and Cancel stops it', (
      tester,
    ) async {
      final (session, gate) = await pumpLock(
        tester,
        summary: const LockSummary(files: 12, bytes: 84000000, shared: 0),
      );
      expect(
        find.textContaining('Its pictures and files (80 MB) are sealed too.'),
        findsOneWidget,
      );
      await tester.tap(find.text('Lock').last);
      await tester.pump(const Duration(milliseconds: 50));
      expect(find.byType(LockProgressSheet), findsOneWidget);

      session.progress.add(
        const SealProgress(
          phase: SealPhase.sealing,
          done: 5,
          total: 12,
          bytesDone: 31000000,
          bytesTotal: 84000000,
        ),
      );
      await tester.pump(const Duration(milliseconds: 50));
      expect(find.text('5 of 12'), findsOneWidget);
      expect(find.text('Sealing 12 attachments, 80 MB'), findsOneWidget);

      await tester.tap(find.text('Cancel'));
      await tester.pump(const Duration(milliseconds: 50));
      expect(session.calls, contains('cancelLock'));
      expect(gate.isCompleted, isTrue);
    });

    testWidgets('a lock that copied files says so as the desktop does', (
      tester,
    ) async {
      final (session, gate) = await pumpLock(
        tester,
        summary: const LockSummary(files: 2, bytes: 1000, shared: 2),
      );
      expect(
        find.textContaining(
          '2 attachments are also used by other notes: the locked note '
          'keeps its own sealed copies, and the other notes keep theirs.',
        ),
        findsOneWidget,
      );
      session.copyNotice = const LockCopyNotice(removed: false, copied: 2);
      gate.complete();
      await tester.tap(find.text('Lock').last);
      await tester.pumpAndSettle();
      expect(
        _heard(),
        'Locked. 2 attachments are also used by other notes, so this note '
        'has its own sealed copies.',
      );
    });

    testWidgets('a short lock shows no sheet of its own', (tester) async {
      final (_, gate) = await pumpLock(
        tester,
        summary: const LockSummary(files: 2, bytes: 1000, shared: 0),
      );
      await tester.tap(find.text('Lock').last);
      await tester.pump(const Duration(milliseconds: 50));
      expect(find.byType(LockProgressSheet), findsNothing);
      gate.complete();
      await tester.pumpAndSettle();
    });
  });

  group('the Remove Lock sheet', () {
    Future<(FixtureSessionRepository, Completer<void>)> pumpRemove(
      WidgetTester tester, {
      LockTarget target = const LockTarget.note('note:1'),
    }) async {
      final session = FixtureSessionRepository(unlocked: true);
      final gate = Completer<void>();
      session.gate = gate;
      addTearDown(session.close);
      registerLockBlocs(session, FixtureDeviceLockRepository());
      final status = LockStatusBloc(session, FixtureDeviceLockRepository());
      addTearDown(status.close);
      await _pumpSheetPage(
        tester,
        BlocProvider<LockStatusBloc>.value(
          value: status,
          child: Builder(
            builder: (context) => TextButton(
              onPressed: () => unawaited(
                RemoveLockSheet.show(context, target, status: status),
              ),
              child: const Text('open'),
            ),
          ),
        ),
      );
      await tester.tap(find.text('open'));
      await tester.pumpAndSettle();
      return (session, gate);
    }

    testWidgets('says what happens to the files, as the desktop does', (
      tester,
    ) async {
      await pumpRemove(tester);
      expect(
        find.textContaining(
          'Its pictures and files are decrypted again too; one that another '
          'locked note also shows is copied, so that note keeps its sealed '
          'copy.',
        ),
        findsOneWidget,
      );
    });

    testWidgets('a notebook says its notes stay locked, with its name', (
      tester,
    ) async {
      await pumpRemove(
        tester,
        target: const LockTarget.notebook('book:1', 'Journal'),
      );
      expect(
        find.textContaining(
          "Notes put in \u201cJournal\u201d from now on aren't locked. The "
          'notes in it now stay locked',
        ),
        findsOneWidget,
      );
    });

    testWidgets('a long Remove Lock counts its files, and Cancel stops it', (
      tester,
    ) async {
      final (session, _) = await pumpRemove(tester);
      await tester.tap(find.text('Remove Lock').last);
      await tester.pump(const Duration(milliseconds: 50));
      session.progress.add(
        const SealProgress(
          phase: SealPhase.sealing,
          done: 5,
          total: 12,
          bytesDone: 31000000,
          bytesTotal: 84000000,
        ),
      );
      await tester.pump(const Duration(milliseconds: 50));
      expect(find.byType(LockProgressSheet), findsOneWidget);
      expect(find.text('Removing lock'), findsOneWidget);
      expect(find.text('Unsealing 12 attachments'), findsOneWidget);
      expect(find.text('5 of 12'), findsOneWidget);

      await tester.tap(find.text('Cancel'));
      await tester.pump(const Duration(milliseconds: 50));
      expect(session.calls, contains('cancelLock'));
      await tester.pumpAndSettle();
      // Stopped before anything changed: the sheet goes, and says so.
      expect(find.byType(LockProgressSheet), findsNothing);
      expect(
        _heard(),
        'Removing the lock stopped. The note is still locked, and its '
        'pictures and files are still sealed.',
      );
    });

    testWidgets('says the copies it made once it is done', (tester) async {
      final (session, gate) = await pumpRemove(tester);
      session.copyNotice = const LockCopyNotice(removed: true, copied: 1);
      gate.complete();
      await tester.tap(find.text('Remove Lock').last);
      await tester.pumpAndSettle();
      expect(
        _heard(),
        'Lock removed. 1 attachment was copied for this note, because '
        'another locked note still shows the sealed one.',
      );
    });
  });
}
