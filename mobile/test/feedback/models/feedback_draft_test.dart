// Kanso — feedback_draft_test.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart'
    show
        Any,
        AnyUtils,
        Generator,
        GeneratorUtils,
        Glados,
        ListAnys,
        StringAnys,
        any;
import 'package:kanso/feedback/models/data/feedback_credit.dart';
import 'package:kanso/feedback/models/data/feedback_draft.dart';
import 'package:kanso/feedback/models/data/feedback_result.dart';
import 'package:kanso/feedback/utils/feedback_labels.dart';
import 'package:kanso/gen/l10n/app_localizations_en.dart';

import '../../helpers/property.dart';

/// Strings near the login rule's edges: its alphabet plus what it refuses.
extension _LoginAny on Any {
  Generator<String> get loginish => listWithLengthInRange(
    0,
    45,
    choose(<String>['a', 'Z', '0', '9', '-', '_', '@', '.', ' ', 'é']),
  ).map((chars) => chars.join());
}

void main() {
  group('FeedbackDraft', () {
    test('Send needs ten characters and no more than 4,000', () {
      expect(const FeedbackDraft(text: '123456789').canSend, isFalse);
      expect(const FeedbackDraft(text: '1234567890').canSend, isTrue);
      expect(FeedbackDraft(text: 'a' * 4000).canSend, isTrue);
      expect(FeedbackDraft(text: 'a' * 4001).canSend, isFalse);
    });

    test('the counter shows from 3,500', () {
      expect(FeedbackDraft(text: 'a' * 3499).showsCounter, isFalse);
      expect(FeedbackDraft(text: 'a' * 3500).showsCounter, isTrue);
    });

    test('a credit name is 1 to 39 letters, numbers, spaces, dots, dashes', () {
      const base = FeedbackDraft(text: 'long enough text', credit: true);
      expect(base.copyWith(creditName: 'tendai').canSend, isTrue);
      expect(base.copyWith(creditName: 'Zoë_1.2-x').canSend, isTrue);
      expect(base.copyWith(creditName: '').canSend, isFalse);
      expect(base.copyWith(creditName: '   ').canSend, isFalse);
      expect(base.copyWith(creditName: '@tendai').canSend, isFalse);
      expect(base.copyWith(creditName: 'a' * 40).canSend, isFalse);
      // Credit off: a bad name doesn't matter.
      expect(
        const FeedbackDraft(
          text: 'long enough text',
          creditName: '@@',
        ).canSend,
        isTrue,
      );
    });

    test('a GitHub credit needs a picked user with a valid login', () {
      const base = FeedbackDraft(
        text: 'long enough text',
        credit: true,
        creditKind: FeedbackCreditKind.github,
      );
      // Nobody picked: not sendable, whatever the name box holds.
      expect(base.canSend, isFalse);
      expect(base.copyWith(creditName: 'tendai').canSend, isFalse);
      const octocat = FeedbackGithubUser(
        login: 'octocat',
        avatarUrl: 'https://avatars.githubusercontent.com/u/583231?v=4',
      );
      expect(base.copyWith(githubUser: octocat).canSend, isTrue);
      expect(
        base.copyWith(githubUser: octocat.copyWith(login: '@octocat')).canSend,
        isFalse,
      );
      // A name credit ignores a leftover pick, and a bad name still blocks.
      expect(
        base
            .copyWith(
              creditKind: FeedbackCreditKind.name,
              githubUser: octocat,
              creditName: '@x',
            )
            .canSend,
        isFalse,
      );
      // Credit off: nothing about it matters.
      expect(
        const FeedbackDraft(
          text: 'long enough text',
          creditKind: FeedbackCreditKind.github,
        ).canSend,
        isTrue,
      );
    });

    Glados<String>(any.letters, kansoExploreConfig()).test(
      'Send is on exactly when the length is in range',
      (text) {
        final draft = FeedbackDraft(text: text);
        expect(
          draft.canSend,
          text.runes.length >= feedbackMinText &&
              text.runes.length <= feedbackMaxText,
        );
      },
    );
  });

  group('GitHub logins', () {
    test('follow GitHub’s rule', () {
      for (final ok in <String>['octocat', 'a', 'a-b', 'A1-b2', 'a' * 39]) {
        expect(isGithubLogin(ok), isTrue, reason: ok);
      }
      for (final bad in <String>[
        '',
        '-a',
        'a-',
        'a--b',
        'a_b',
        'a b',
        '@a',
        '名前',
        'a' * 40,
      ]) {
        expect(isGithubLogin(bad), isFalse, reason: bad);
      }
    });

    // Oracle: the rule stated by segments, not by pattern: runs of ASCII
    // letters and digits joined by single hyphens, 1 to 39 long.
    Glados<String>(any.loginish, kansoExploreConfig()).test(
      'agree with the segment form on near-miss strings',
      (text) {
        final bySegments =
            text.isNotEmpty &&
            text.length <= 39 &&
            text
                .split('-')
                .every(
                  (part) =>
                      part.isNotEmpty &&
                      part.codeUnits.every(
                        (c) =>
                            (c >= 0x30 && c <= 0x39) ||
                            (c >= 0x41 && c <= 0x5a) ||
                            (c >= 0x61 && c <= 0x7a),
                      ),
                );
        expect(isGithubLogin(text), bySegments);
      },
    );

    test('a search query is 2 to 39 login characters, minus a leading @', () {
      expect(githubSearchQuery('octo'), 'octo');
      expect(githubSearchQuery('  @Octo  '), 'Octo');
      expect(githubSearchQuery('o'), isNull);
      expect(githubSearchQuery(''), isNull);
      expect(githubSearchQuery('a b'), isNull);
      expect(githubSearchQuery('octo in:login'), isNull);
      expect(githubSearchQuery('名前名前'), isNull);
      expect(githubSearchQuery('a' * 40), isNull);
      expect(githubSearchQuery('a' * 39), 'a' * 39);
    });

    // Refinement: whatever is searched is only login characters.
    Glados<String>(any.loginish, kansoExploreConfig()).test(
      'a search query only ever holds login characters',
      (text) {
        final query = githubSearchQuery(text);
        if (query != null) {
          expect(RegExp(r'^[A-Za-z0-9-]{2,39}$').hasMatch(query), isTrue);
          expect(text.trim().replaceFirst(RegExp('^@'), ''), query);
        }
      },
    );
  });

  group('FeedbackLabels', () {
    final l10n = AppLocalizationsEn();

    test("says the spec's words", () {
      expect(
        l10n.feedbackProblemMessage(FeedbackProblem.tooShort),
        'Say a little more (at least 10 characters).',
      );
      expect(
        l10n.feedbackProblemMessage(FeedbackProblem.tooLong),
        'Keep it under 4,000 characters.',
      );
      expect(
        l10n.feedbackOutcomeMessage(
          const FeedbackSendResult(outcome: FeedbackOutcome.sent),
        ),
        'Thanks — feedback sent.',
      );
      expect(
        l10n.feedbackOutcomeMessage(
          const FeedbackSendResult(outcome: FeedbackOutcome.queued),
        ),
        'Saved — it will send when you’re back online.',
      );
      expect(
        l10n.feedbackOutcomeMessage(
          const FeedbackSendResult(outcome: FeedbackOutcome.rateLimited),
        ),
        'You’ve sent a lot of feedback today. Try again tomorrow.',
      );
    });

    test('says to pick a GitHub user', () {
      expect(
        l10n.feedbackProblemMessage(FeedbackProblem.badGithubUser),
        'Pick your GitHub username from the list.',
      );
    });

    test('a refusal names the field and never anything else', () {
      expect(
        l10n.feedbackRejectedMessage('app.version'),
        'Kanso couldn’t send this report (app.version). Edit it and try again.',
      );
      expect(
        l10n.feedbackRejectedMessage('<b>x</b> "quoted" value'),
        isNot(contains('"')),
      );
      expect(
        l10n.feedbackRejectedMessage(''),
        'Kanso couldn’t send this report. Edit it and try again.',
      );
    });

    Glados<String>(any.letterOrDigits, kansoExploreConfig()).test(
      'a refusal shows at most 40 safe characters of the field',
      (field) {
        final message = l10n.feedbackRejectedMessage('$field<>"\' ');
        final inner = RegExp(r'\((.*)\)').firstMatch(message)?.group(1) ?? '';
        expect(inner.length, lessThanOrEqualTo(40));
        expect(RegExp(r'^[A-Za-z0-9._\[\]]*$').hasMatch(inner), isTrue);
      },
    );

    test('only a kept report closes the page', () {
      expect(feedbackClosesPage(FeedbackOutcome.sent), isTrue);
      expect(feedbackClosesPage(FeedbackOutcome.queued), isTrue);
      expect(feedbackClosesPage(FeedbackOutcome.paused), isTrue);
      expect(feedbackClosesPage(FeedbackOutcome.rateLimited), isTrue);
      expect(feedbackClosesPage(FeedbackOutcome.rejected), isFalse);
      expect(feedbackClosesPage(FeedbackOutcome.invalid), isFalse);
    });
  });
}
