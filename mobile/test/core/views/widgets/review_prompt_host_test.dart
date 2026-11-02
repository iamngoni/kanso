// Kanso — review_prompt_host_test.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:clock/clock.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/review_moment.dart';
import 'package:kanso/core/models/data/review_prompt_record.dart';
import 'package:kanso/core/services/review_prompt.dart';
import 'package:kanso/core/state/sync_status/sync_status_bloc.dart';
import 'package:kanso/core/views/widgets/review_prompt_host.dart';
import 'package:kanso/crash/models/data/pending_crash_report.dart';

import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/crash_samples.dart';
import '../../../helpers/fixture_about_repository.dart';
import '../../../helpers/fixture_sync_repository.dart';
import '../../../helpers/review_prompt_fixtures.dart';

const SyncStatus _synced = SyncStatus(
  phase: SyncPhase.synced,
  username: 'ana',
  conflicts: 0,
  skippedCount: 0,
  account: SyncAccount.signedIn(),
);

/// A signed-in status in [phase], with [problem] and [account].
SyncStatus _status({
  SyncPhase phase = SyncPhase.synced,
  SyncProblemKind? problem,
  SyncAccount account = const SyncAccount.signedIn(),
}) => SyncStatus(
  phase: phase,
  username: 'ana',
  conflicts: 0,
  skippedCount: 0,
  problem: problem,
  account: account,
);

class _Host {
  _Host(this.router, this.records, this.store, this.prompt);

  final GoRouter router;
  final MemoryReviewPromptRepository records;
  final FixtureStoreReviewRepository store;
  final ReviewPrompt prompt;
}

/// The host over a list (`/notes`) and a note (`/notes/note/:id`), with a
/// person who qualifies, and [keyboard] as the keyboard's height.
Future<_Host> _pump(
  WidgetTester tester, {
  SyncStatus status = signedOutStatus,
  double keyboard = 0,
  String start = '/notes',
}) async {
  final records = MemoryReviewPromptRepository(
    ReviewPromptRecord(
      firstOpenedAt: clock.now().subtract(const Duration(days: 30)),
      activeDays: 12,
      notesEdited: 40,
    ),
  );
  final store = FixtureStoreReviewRepository();
  final prompt = ReviewPrompt(
    records,
    store,
    FixtureAboutRepository(),
    FixtureCrashReportRepository(<PendingCrashReport>[]),
  );
  final router = GoRouter(
    initialLocation: start,
    routes: <RouteBase>[
      GoRoute(
        path: '/notes',
        builder: (_, _) => const SizedBox.expand(),
        routes: <RouteBase>[
          GoRoute(path: 'note/:id', builder: (_, _) => const SizedBox.expand()),
        ],
      ),
    ],
  );
  addTearDown(router.dispose);
  final sync = SyncStatusBloc(FixtureSyncRepository(current: status))
    ..add(const SyncStatusEvent.started());
  closeBlocLater(sync);
  await tester.pumpWidget(
    MediaQuery(
      data: MediaQueryData(viewInsets: EdgeInsets.only(bottom: keyboard)),
      child: BlocProvider<SyncStatusBloc>.value(
        value: sync,
        child: ReviewPromptHost(
          prompt: prompt,
          router: router,
          child: WidgetsApp.router(
            color: const Color(0xFF000000),
            routerConfig: router,
          ),
        ),
      ),
    ),
  );
  await tester.pump();
  return _Host(router, records, store, prompt);
}

void main() {
  group('ReviewPromptHost', () {
    testWidgets('counts the app as opened today', (tester) async {
      final host = await _pump(tester);
      expect(host.records.record.lastActiveDay, isNotNull);
      expect(host.records.record.activeDays, 13);
    });

    testWidgets('asks on a list, keyboard down, after a good moment', (
      tester,
    ) async {
      final host = await _pump(tester);
      host.prompt.moment(ReviewMoment.taskDone);
      await tester.pump(reviewSettle ~/ 2);
      expect(host.store.requests, 0, reason: 'not before it settles');
      await tester.pump(reviewSettle);
      await tester.pump();
      expect(host.store.requests, 1);
    });

    testWidgets('waits until the person is back on a list', (tester) async {
      final host = await _pump(tester, start: '/notes/note/n1');
      host.prompt.moment(ReviewMoment.published);
      await tester.pump(reviewSettle * 2);
      expect(host.store.requests, 0, reason: 'never over a note');
      host.router.go('/notes');
      await tester.pumpAndSettle();
      await tester.pump(reviewSettle);
      await tester.pump();
      expect(host.store.requests, 1);
    });

    testWidgets('never asks over a sheet or dialog on a list', (tester) async {
      final host = await _pump(tester);
      final context = host.router.routerDelegate.navigatorKey.currentContext!;
      unawaited(
        Navigator.of(context).push<void>(
          PageRouteBuilder<void>(
            opaque: false,
            pageBuilder: (_, _, _) => const SizedBox.expand(),
          ),
        ),
      );
      await tester.pumpAndSettle();
      host.prompt.moment(ReviewMoment.taskDone);
      await tester.pump(reviewSettle * 2);
      expect(host.store.requests, 0, reason: 'a modal route is on top');
      Navigator.of(context).pop();
      await tester.pumpAndSettle();
      host.prompt.moment(ReviewMoment.taskDone);
      await tester.pump(reviewSettle);
      await tester.pump();
      expect(host.store.requests, 1);
    });

    testWidgets('never asks while the keyboard is up', (tester) async {
      final host = await _pump(tester, keyboard: 300);
      host.prompt.moment(ReviewMoment.taskDone);
      await tester.pump(reviewSettle * 2);
      expect(host.store.requests, 0);
    });

    testWidgets('the first sync is a good moment', (tester) async {
      final host = await _pump(tester, status: _synced);
      await tester.pump(reviewSettle);
      await tester.pump();
      expect(host.records.record.firstSyncSeen, isTrue);
      expect(host.store.requests, 1);
    });

    testWidgets('a failed sync keeps it quiet', (tester) async {
      final host = await _pump(
        tester,
        status: _status(phase: SyncPhase.failed),
      );
      expect(host.records.record.lastTroubleAt, isNotNull);
      host.prompt.moment(ReviewMoment.taskDone);
      await tester.pump(reviewSettle * 2);
      expect(host.store.requests, 0);
    });
  });

  group('isReviewSyncTrouble', () {
    test('is a failure, an ended session or a problem with the account', () {
      expect(isReviewSyncTrouble(const SyncStatusState()), isFalse);
      expect(
        isReviewSyncTrouble(const SyncStatusState(status: _synced)),
        isFalse,
      );
      expect(
        isReviewSyncTrouble(
          const SyncStatusState(
            error: ApplicationError.unexpected(message: 'No'),
          ),
        ),
        isTrue,
      );
      expect(
        isReviewSyncTrouble(
          SyncStatusState(status: _status(phase: SyncPhase.failed)),
        ),
        isTrue,
      );
      expect(
        isReviewSyncTrouble(
          SyncStatusState(
            status: _status(account: const SyncAccount.sessionEnded()),
          ),
        ),
        isTrue,
      );
      for (final problem in <SyncProblemKind>[
        SyncProblemKind.signedOut,
        SyncProblemKind.accountDisabled,
        SyncProblemKind.keysDontMatch,
      ]) {
        expect(
          isReviewSyncTrouble(
            SyncStatusState(status: _status(problem: problem)),
          ),
          isTrue,
          reason: '$problem',
        );
      }
      for (final problem in <SyncProblemKind>[
        SyncProblemKind.offline,
        SyncProblemKind.paused,
        SyncProblemKind.backOff,
      ]) {
        expect(
          isReviewSyncTrouble(
            SyncStatusState(status: _status(problem: problem)),
          ),
          isFalse,
          reason: '$problem',
        );
      }
    });
  });
}
