// Kanso — sharing_utils_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart'
    show Glados, Glados2, Glados3, IntAnys, StringAnys, any;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/sharing/models/data/share_problem.dart';
import 'package:kanso/sharing/utils/days_until.dart';
import 'package:kanso/sharing/utils/member_status.dart';
import 'package:kanso/sharing/utils/share_problem_of.dart';

import '../../helpers/property.dart';
import '../../helpers/sharing_fixtures.dart';
import '../../helpers/sharing_property.dart';

void main() {
  group('memberStatus', () {
    Glados<MemberRow>(any.member, kansoExploreConfig()).test(
      'someone who left says only that, dimmed, and "Declined" if they never '
      'accepted',
      (member) {
        final status = memberStatus(member);
        if (member.left) {
          expect(status.tone, MemberTone.gone);
          final gone = member.invited ? MemberNote.declined : MemberNote.left;
          expect(status.notes, <MemberNote>[gone]);
        } else {
          expect(status.tone, isNot(MemberTone.gone));
          expect(status.notes, isNot(contains(MemberNote.left)));
          expect(status.notes, isNot(contains(MemberNote.declined)));
        }
      },
    );

    Glados<MemberRow>(any.member, kansoExploreConfig()).test(
      'a warning is only for someone else whose keys changed, and it never '
      'says the fingerprint is verified',
      (member) {
        final status = memberStatus(member);
        final warned = status.tone == MemberTone.unchecked;
        expect(
          warned,
          !member.left && !member.me && member.keyChanged,
        );
        if (warned) {
          expect(status.notes.last, MemberNote.unchecked);
          expect(status.notes, isNot(contains(MemberNote.verified)));
        }
      },
    );

    Glados<MemberRow>(any.member, kansoExploreConfig()).test(
      'a person’s own row says "You", and an invitation not accepted yet '
      'says so first',
      (member) {
        final notes = memberStatus(member).notes;
        if (!member.left) {
          expect(notes.contains(MemberNote.you), member.me);
          expect(notes.contains(MemberNote.invited), member.invited);
          if (member.invited) {
            expect(notes.first, MemberNote.invited);
          }
        }
      },
    );

    test('reads as the desktop’s examples do', () {
      expect(
        memberStatus(memberRow('ana', invited: true, verified: true)).notes,
        <MemberNote>[MemberNote.invited, MemberNote.verified],
      );
      expect(
        memberStatus(memberRow('ana', keyChanged: true)).notes,
        <MemberNote>[MemberNote.unchecked],
      );
      expect(memberStatus(memberRow('ana')).notes, isEmpty);
    });
  });

  group('daysUntil', () {
    Glados2<int, int>(
      any.intInRange(0, 1 << 40),
      any.intInRange(-(1 << 30), 1 << 40),
      kansoExploreConfig(),
    ).test('is the whole days left, rounded up', (now, ahead) {
      final days = daysUntil(now + ahead, now);
      if (ahead <= 0) {
        expect(days, 0);
      } else {
        expect(days * millisecondsPerDay, greaterThanOrEqualTo(ahead));
        expect((days - 1) * millisecondsPerDay, lessThan(ahead));
      }
    });

    Glados3<int, int, int>(
      any.intInRange(0, 1 << 40),
      any.intInRange(0, 1 << 30),
      any.intInRange(0, 1 << 30),
      kansoExploreConfig(),
    ).test('never goes up as time goes on', (now, a, b) {
      final early = daysUntil(now + 1000000000, now + a);
      final late = daysUntil(now + 1000000000, now + a + b);
      expect(late, lessThanOrEqualTo(early));
    });

    test('an hour left is a day left, and a day exactly is a day', () {
      expect(daysUntil(3600000, 0), 1);
      expect(daysUntil(millisecondsPerDay, 0), 1);
      expect(daysUntil(millisecondsPerDay + 1, 0), 2);
      expect(daysUntil(5, 5), 0);
    });
  });

  group('shareProblemOf', () {
    test('shows the library’s sentence, trimmed, for what it refuses', () {
      expect(
        shareProblemOf(
          const ApplicationError.invalid(message: ' Only the owner can. '),
        ),
        const ShareProblem.words('Only the owner can.'),
      );
      expect(
        shareProblemOf(const ApplicationError.auth(message: 'Turned off.')),
        const ShareProblem.words('Turned off.'),
      );
    });

    test('has words of its own for the network and a signed-out device', () {
      expect(
        shareProblemOf(const ApplicationError.network(message: 'dns')),
        const ShareProblem.offline(),
      );
      expect(
        shareProblemOf(const ApplicationError.signedOut(message: 'x')),
        const ShareProblem.signedOut(),
      );
    });

    test('says nothing of a bug or a broken disk', () {
      for (final error in const <ApplicationError>[
        ApplicationError.internal(message: 'secret detail'),
        ApplicationError.storage(message: 'secret detail'),
        ApplicationError.unexpected(message: 'secret detail'),
        ApplicationError.notFound(message: 'secret detail'),
        ApplicationError.locked(message: 'secret detail'),
        ApplicationError.conflict(message: 'secret detail'),
        ApplicationError.invalid(message: '   '),
      ]) {
        expect(shareProblemOf(error), const ShareProblem.unexpected());
      }
    });

    Glados<String>(any.letterOrDigits, kansoExploreConfig()).test(
      'a problem never shows a message that is not a sentence for people',
      (message) {
        for (final error in <ApplicationError>[
          ApplicationError.internal(message: message),
          ApplicationError.network(message: message),
          ApplicationError.storage(message: message),
        ]) {
          final problem = shareProblemOf(error);
          expect(problem is ShareProblemWords, isFalse);
        }
      },
    );
  });
}
