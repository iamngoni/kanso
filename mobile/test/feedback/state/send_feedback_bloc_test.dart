// Kanso — send_feedback_bloc_test.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:typed_data';

import 'package:bloc_test/bloc_test.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/feedback/models/data/feedback_credit.dart';
import 'package:kanso/feedback/models/data/feedback_draft.dart';
import 'package:kanso/feedback/models/data/feedback_kind.dart';
import 'package:kanso/feedback/models/data/feedback_result.dart';
import 'package:kanso/feedback/state/send_feedback/send_feedback_bloc.dart';

import '../../helpers/fixture_feedback_repository.dart';

void main() {
  late FixtureFeedbackRepository repository;
  late FixtureFeedbackScreenshots screenshots;

  setUp(() {
    repository = FixtureFeedbackRepository();
    screenshots = FixtureFeedbackScreenshots();
  });

  SendFeedbackBloc build() => SendFeedbackBloc(repository, screenshots);

  group('SendFeedbackBloc', () {
    blocTest<SendFeedbackBloc, SendFeedbackState>(
      'typing builds the draft and Send sends exactly that',
      build: build,
      act: (bloc) {
        bloc
          ..add(const SendFeedbackEvent.kindChanged(FeedbackKind.idea))
          ..add(const SendFeedbackEvent.textChanged('Dark mode for the graph'))
          ..add(const SendFeedbackEvent.sendRequested());
      },
      wait: const Duration(milliseconds: 20),
      verify: (bloc) {
        expect(repository.sent, hasLength(1));
        expect(repository.sent.single.kind, FeedbackKind.idea);
        expect(repository.sent.single.text, 'Dark mode for the graph');
        expect(bloc.state.result?.outcome, FeedbackOutcome.sent);
        expect(bloc.state.sending, isFalse);
      },
    );

    blocTest<SendFeedbackBloc, SendFeedbackState>(
      'a draft under ten characters is not sent',
      build: build,
      act: (bloc) {
        bloc
          ..add(const SendFeedbackEvent.textChanged('short'))
          ..add(const SendFeedbackEvent.sendRequested());
      },
      wait: const Duration(milliseconds: 20),
      verify: (bloc) {
        expect(repository.sent, isEmpty);
        expect(bloc.state.result, isNull);
      },
    );

    blocTest<SendFeedbackBloc, SendFeedbackState>(
      'a report saved for later is told as queued',
      build: () {
        repository.result = const FeedbackSendResult(
          outcome: FeedbackOutcome.queued,
        );
        return build();
      },
      act: (bloc) {
        bloc
          ..add(const SendFeedbackEvent.textChanged('Export cuts off tables'))
          ..add(const SendFeedbackEvent.sendRequested());
      },
      wait: const Duration(milliseconds: 20),
      verify: (bloc) =>
          expect(bloc.state.result?.outcome, FeedbackOutcome.queued),
    );

    blocTest<SendFeedbackBloc, SendFeedbackState>(
      'a 429 is told and the report is not lost',
      build: () {
        repository.result = const FeedbackSendResult(
          outcome: FeedbackOutcome.rateLimited,
        );
        return build();
      },
      act: (bloc) {
        bloc
          ..add(const SendFeedbackEvent.textChanged('Export cuts off tables'))
          ..add(const SendFeedbackEvent.sendRequested());
      },
      wait: const Duration(milliseconds: 20),
      verify: (bloc) {
        expect(repository.sent, hasLength(1));
        expect(bloc.state.result?.outcome, FeedbackOutcome.rateLimited);
      },
    );

    blocTest<SendFeedbackBloc, SendFeedbackState>(
      'a 400 keeps the page with the field',
      build: () {
        repository.result = const FeedbackSendResult(
          outcome: FeedbackOutcome.rejected,
          field: 'text',
        );
        return build();
      },
      act: (bloc) {
        bloc
          ..add(const SendFeedbackEvent.textChanged('Export cuts off tables'))
          ..add(const SendFeedbackEvent.sendRequested());
      },
      wait: const Duration(milliseconds: 20),
      verify: (bloc) {
        expect(bloc.state.result?.outcome, FeedbackOutcome.rejected);
        expect(bloc.state.result?.field, 'text');
      },
    );

    blocTest<SendFeedbackBloc, SendFeedbackState>(
      "Show what's sent follows the text as it is typed",
      build: build,
      act: (bloc) {
        bloc
          ..add(const SendFeedbackEvent.sentToggled())
          ..add(const SendFeedbackEvent.textChanged('Export cuts off tables'));
      },
      wait: const Duration(milliseconds: 20),
      verify: (bloc) {
        expect(bloc.state.showSent, isTrue);
        expect(bloc.state.payload.json, contains('Export cuts off tables'));
        expect(bloc.state.payload.problem, isNull);
      },
    );

    blocTest<SendFeedbackBloc, SendFeedbackState>(
      'a photo is attached, and removed',
      build: () {
        screenshots.picked = FeedbackScreenshot(
          bytes: Uint8List.fromList(<int>[1, 2, 3]),
          name: 'shot.png',
        );
        return build();
      },
      act: (bloc) async {
        bloc.add(const SendFeedbackEvent.screenshotRequested());
        await bloc.stream.firstWhere((s) => s.draft.screenshot != null);
        bloc.add(const SendFeedbackEvent.screenshotRemoved());
      },
      wait: const Duration(milliseconds: 20),
      verify: (bloc) {
        expect(screenshots.picks, 1);
        expect(bloc.state.draft.screenshot, isNull);
      },
    );

    blocTest<SendFeedbackBloc, SendFeedbackState>(
      'a photo over 5 MB is not attached',
      build: () {
        screenshots.picked = FeedbackScreenshot(
          bytes: Uint8List(feedbackMaxScreenshot + 1),
          name: 'big.png',
        );
        return build();
      },
      act: (bloc) => bloc.add(const SendFeedbackEvent.screenshotRequested()),
      wait: const Duration(milliseconds: 20),
      verify: (bloc) {
        expect(bloc.state.screenshotTooBig, isTrue);
        expect(bloc.state.draft.screenshot, isNull);
      },
    );
  });

  group('SendFeedbackBloc GitHub credit', () {
    const octocat = FeedbackGithubUser(
      login: 'octocat',
      avatarUrl: 'https://avatars.githubusercontent.com/u/583231?v=4',
    );
    const wait = Duration(milliseconds: 450);

    blocTest<SendFeedbackBloc, SendFeedbackState>(
      'typing searches once, after the pause, and shows the users',
      build: () {
        repository.users = const <FeedbackGithubUser>[octocat];
        return build();
      },
      act: (bloc) {
        bloc
          ..add(const SendFeedbackEvent.githubQueryChanged('oc'))
          ..add(const SendFeedbackEvent.githubQueryChanged('octo'));
      },
      wait: wait,
      verify: (bloc) {
        expect(repository.searches, <String>['octo']);
        expect(
          bloc.state.githubSearch,
          const FeedbackGithubSearch.found(<FeedbackGithubUser>[octocat]),
        );
      },
    );

    blocTest<SendFeedbackBloc, SendFeedbackState>(
      'a query too short to search asks nothing',
      build: build,
      act: (bloc) => bloc.add(const SendFeedbackEvent.githubQueryChanged('o')),
      wait: wait,
      verify: (bloc) {
        expect(repository.searches, isEmpty);
        expect(bloc.state.githubSearch, const FeedbackGithubSearch.idle());
      },
    );

    blocTest<SendFeedbackBloc, SendFeedbackState>(
      'a search that cannot be made says unavailable',
      build: () {
        repository.users = null;
        return build();
      },
      act: (bloc) =>
          bloc.add(const SendFeedbackEvent.githubQueryChanged('octo')),
      wait: wait,
      verify: (bloc) => expect(
        bloc.state.githubSearch,
        const FeedbackGithubSearch.unavailable(),
      ),
    );

    blocTest<SendFeedbackBloc, SendFeedbackState>(
      'a GitHub credit waits for a pick, then sends the login',
      build: build,
      act: (bloc) async {
        bloc
          ..add(const SendFeedbackEvent.textChanged('Export cuts off tables'))
          ..add(const SendFeedbackEvent.creditToggled(on: true))
          ..add(
            const SendFeedbackEvent.creditKindChanged(
              FeedbackCreditKind.github,
            ),
          );
        await Future<void>.delayed(const Duration(milliseconds: 20));
        expect(bloc.state.draft.canSend, isFalse);
        bloc
          ..add(const SendFeedbackEvent.githubUserPicked(octocat))
          ..add(const SendFeedbackEvent.sendRequested());
      },
      wait: const Duration(milliseconds: 40),
      verify: (bloc) {
        expect(repository.sent, hasLength(1));
        expect(repository.sent.single.creditKind, FeedbackCreditKind.github);
        expect(repository.sent.single.githubUser?.login, 'octocat');
        expect(bloc.state.githubSearch, const FeedbackGithubSearch.idle());
      },
    );

    blocTest<SendFeedbackBloc, SendFeedbackState>(
      'Change takes the chosen user back',
      build: build,
      act: (bloc) => bloc
        ..add(const SendFeedbackEvent.githubUserPicked(octocat))
        ..add(const SendFeedbackEvent.githubUserCleared()),
      wait: const Duration(milliseconds: 20),
      verify: (bloc) => expect(bloc.state.draft.githubUser, isNull),
    );
  });
}
