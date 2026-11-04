// Kanso — feedback_suite.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/feedback.dart';

import 'open_library.dart';

/// Send feedback and My feedback through the bridge (issue #232). It runs
/// on this computer (`test/core/bridge`) and on a simulator or an emulator
/// (`integration_test`).
///
/// Nothing here reaches Kanso's server: the reports are invalid, so `send`
/// stops before the network; the outbox is empty, so `flush` has nothing to
/// send; `reports` is read without a refresh; and the report deleted was
/// never on the server.
void feedbackSuite(OpenLibrary open) {
  const context = FeedbackContext(
    version: '0.9.4+57',
    build: '57',
    os: 'iOS 18.1',
    device: 'iPhone16,1',
    locale: 'en_GB',
    screen: 'settings',
    crashTypes: <String>[],
  );
  FeedbackInput input(String text, {bool includeApp = true}) => FeedbackInput(
    kind: 'bug',
    text: text,
    includeApp: includeApp,
    context: context,
  );

  group('KansoFeedback', () {
    test('the preview shows what is sent, or why nothing is', () async {
      final feedback = (await open()).feedback();

      final ok = await feedback.preview(
        input: input('The Overview page forgets the week I was on.'),
      );
      expect(ok.problem, isNull);
      expect(ok.json, contains('forgets the week'));
      expect(ok.json, contains('0.9.4'));

      final bare = await feedback.preview(
        input: input(
          'The Overview page forgets the week I was on.',
          includeApp: false,
        ),
      );
      expect(bare.problem, isNull);
      expect(bare.json, isNot(contains('iPhone16,1')));

      final short = await feedback.preview(input: input('too short'));
      expect(short.problem, FeedbackProblem.tooShort);
      expect(short.json, isEmpty);
    });

    test('an invalid report is refused before anything is sent', () async {
      final feedback = (await open()).feedback();
      final sent = await feedback.send(input: input('short'));
      expect(
        sent,
        const FeedbackSent.invalid(problem: FeedbackProblem.tooShort),
      );
      expect(await feedback.reports(refresh: false), isEmpty);
    });

    test('an empty outbox flushes, retries and lists nothing', () async {
      final feedback = (await open()).feedback();
      expect(feedback.refreshDue(), isTrue);
      await feedback.flush();
      await feedback.retryNow();
      await feedback.markSeen();
      await feedback.delete(id: 'never-sent');
      expect(await feedback.reports(refresh: false), isEmpty);
    });
  });
}
