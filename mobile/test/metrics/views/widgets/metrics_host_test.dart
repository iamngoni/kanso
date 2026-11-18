// Kanso — metrics_host_test.dart
// Created 2026-10-31.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/ai_settings/state/ai_status/ai_status_bloc.dart';
import 'package:kanso/metrics/views/widgets/metrics_host.dart';

import '../../../helpers/fake_usage_metrics.dart';
import '../../../helpers/fixture_ai_repository.dart';
import '../../../helpers/kanso_test_app.dart';

void main() {
  Future<AiStatusBloc> pumpHost(
    WidgetTester tester,
    FakeUsageMetrics metrics, {
    Duration firstSendDelay = const Duration(seconds: 20),
    DateTime Function()? clock,
  }) async {
    final ai = AiStatusBloc(FixtureAiRepository());
    addTearDown(ai.close);
    await tester.pumpWidget(
      KansoTestApp(
        home: BlocProvider<AiStatusBloc>.value(
          value: ai,
          child: MetricsHost(
            metrics: metrics,
            firstSendDelay: firstSendDelay,
            clock: clock ?? DateTime.now,
            child: const Scaffold(body: Text('the app')),
          ),
        ),
      ),
    );
    // The first read starts after the first frame.
    await tester.pump();
    await tester.pump();
    return ai;
  }

  testWidgets('shows the notice and sends only after OK', (tester) async {
    final metrics = FakeUsageMetrics();
    await pumpHost(tester, metrics);

    expect(find.text('Kanso counts, it doesn’t watch'), findsOneWidget);
    expect(metrics.sendDueCalls, 0);

    await tester.tap(find.text('OK'));
    await tester.pump();
    await tester.pump();

    expect(metrics.markSeenCalls, 1);
    expect(metrics.sendDueCalls, 1);
    expect(metrics.launchedCalls, 1);
    expect(metrics.sends.single.osVersion, isNotEmpty);
    expect(metrics.sends.single.locale, 'en');

    // Let the host dispose its hourly timer.
    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('a notice already seen sends after the settle delay', (
    tester,
  ) async {
    final metrics = FakeUsageMetrics(seen: true);
    await pumpHost(
      tester,
      metrics,
      firstSendDelay: const Duration(milliseconds: 20),
    );

    expect(find.text('Kanso counts, it doesn’t watch'), findsNothing);
    expect(metrics.sendDueCalls, 0);

    await tester.pump(const Duration(milliseconds: 21));
    await tester.pump();

    expect(metrics.sendDueCalls, 1);
    expect(metrics.markSeenCalls, 0);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('a failed notice read shows nothing and sends nothing', (
    tester,
  ) async {
    final metrics = FakeUsageMetrics(noticeSeenThrows: true);
    await pumpHost(tester, metrics);

    expect(find.text('Kanso counts, it doesn’t watch'), findsNothing);
    expect(metrics.sendDueCalls, 0);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('a long stay away ends the session when the app is back', (
    tester,
  ) async {
    var now = DateTime.utc(2026, 11, 1, 9);
    final metrics = FakeUsageMetrics(seen: true);
    await pumpHost(
      tester,
      metrics,
      firstSendDelay: const Duration(hours: 5),
      clock: () => now,
    );
    expect(metrics.launchedCalls, 1);

    // Going to the background sends, but the session stays open.
    now = now.add(const Duration(minutes: 7, seconds: 3));
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
    await tester.pump();
    await tester.pump();
    expect(metrics.sessions, isEmpty);
    expect(metrics.calls, <String>['launched', 'sendDue']);

    // Going back and forth without being in front again changes nothing.
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.hidden);
    await tester.pump();
    expect(metrics.sessions, isEmpty);

    // Back after five minutes: the session ends with its time in front,
    // and the next stretch is a session of its own.
    now = now.add(const Duration(minutes: 5));
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
    await tester.pump();
    await tester.pump();
    expect(metrics.sessions, <int>[423]);
    expect(metrics.calls.last, 'sendDue');

    now = now.add(const Duration(seconds: 30));
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.detached);
    await tester.pump();
    await tester.pump();
    expect(metrics.sessions, <int>[423, 30]);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('short trips away continue one session', (tester) async {
    var now = DateTime.utc(2026, 11, 1, 9);
    final metrics = FakeUsageMetrics(seen: true);
    await pumpHost(
      tester,
      metrics,
      firstSendDelay: const Duration(hours: 5),
      clock: () => now,
    );

    // Three photo-picker round trips: 20 s in front, 40 s away, each time.
    for (var i = 0; i < 3; i++) {
      now = now.add(const Duration(seconds: 20));
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
      await tester.pump();
      now = now.add(const Duration(seconds: 40));
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
      await tester.pump();
    }
    expect(metrics.sessions, isEmpty);

    // Away for exactly the grace period still continues it.
    now = now.add(const Duration(seconds: 10));
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
    await tester.pump();
    now = now.add(const Duration(seconds: 60));
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
    await tester.pump();
    expect(metrics.sessions, isEmpty);

    // Detached ends it: only the time in front counts.
    now = now.add(const Duration(seconds: 5));
    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.detached);
    await tester.pump();
    await tester.pump();
    expect(metrics.sessions, <int>[75]);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('nothing is noted before the notice was acknowledged', (
    tester,
  ) async {
    final metrics = FakeUsageMetrics();
    await pumpHost(tester, metrics);

    tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
    await tester.pump();

    expect(metrics.launchedCalls, 0);
    expect(metrics.sessions, isEmpty);
    expect(metrics.sendDueCalls, 0);

    await tester.pumpWidget(const SizedBox.shrink());
  });
}
