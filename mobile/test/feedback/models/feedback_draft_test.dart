// Kanso — feedback_draft_test.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados, StringAnys, any;
import 'package:kanso/feedback/models/data/feedback_draft.dart';
import 'package:kanso/feedback/models/data/feedback_result.dart';
import 'package:kanso/feedback/utils/feedback_labels.dart';
import 'package:kanso/gen/l10n/app_localizations_en.dart';

import '../../helpers/property.dart';

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
