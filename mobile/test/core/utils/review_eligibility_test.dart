// Kanso — review_eligibility_test.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart'
    show Any, BoolAny, CombinableAny, Generator, Glados, IntAnys, any;
import 'package:kanso/core/models/data/review_prompt_record.dart';
import 'package:kanso/core/utils/review_eligibility.dart';

import '../../helpers/property.dart';

final DateTime _now = DateTime.utc(2026, 11, 2, 12);

DateTime _daysAgo(int days) => _now.subtract(Duration(days: days));

/// Someone who has used the app for a month, on many days, with many notes,
/// and has never been asked.
final ReviewPromptRecord _happy = ReviewPromptRecord(
  firstOpenedAt: _daysAgo(30),
  activeDays: 12,
  lastActiveDay: '2026-11-02',
  notesEdited: 40,
);

ReviewIneligibility? _why(
  ReviewPromptRecord record, {
  String version = '1.2.0',
}) => reviewIneligibility(record, now: _now, version: version);

/// A record whose times are up to [_span] days either side of now (null
/// when the number drawn is below zero), with counts around the thresholds
/// and an asked version drawn from three.
extension on Any {
  Generator<ReviewPromptRecord> get reviewRecord => combine7(
    intInRange(-1, _span),
    intInRange(0, 12),
    intInRange(0, 25),
    intInRange(-1, _span),
    intInRange(-1, _span),
    intInRange(0, 4),
    this.bool,
    (first, days, notes, trouble, asked, version, synced) => ReviewPromptRecord(
      firstOpenedAt: first < 0 ? null : _daysAgo(first - 20),
      activeDays: days,
      notesEdited: notes,
      lastTroubleAt: trouble < 0 ? null : _daysAgo(trouble - 20),
      lastAskedAt: asked < 0 ? null : _daysAgo(asked - 20),
      askedVersions: <String>[if (version > 0) '1.$version.0'],
      firstSyncSeen: synced,
    ),
  );
}

const int _span = 200;

void main() {
  group('reviewIneligibility', () {
    test('lets a happy person who was never asked be asked', () {
      expect(_why(_happy), isNull);
      expect(reviewEligible(_happy, now: _now, version: '1.2.0'), isTrue);
    });

    test('a fresh install is never asked in its first week', () {
      expect(_why(const ReviewPromptRecord()), ReviewIneligibility.tooNew);
      for (final days in <int>[0, 1, 6]) {
        expect(
          _why(_happy.copyWith(firstOpenedAt: _daysAgo(days))),
          ReviewIneligibility.tooNew,
          reason: '$days days',
        );
      }
      expect(_why(_happy.copyWith(firstOpenedAt: _daysAgo(7))), isNull);
    });

    test('needs five days of use and ten notes', () {
      expect(
        _why(_happy.copyWith(activeDays: 4)),
        ReviewIneligibility.tooFewDays,
      );
      expect(_why(_happy.copyWith(activeDays: 5)), isNull);
      expect(
        _why(_happy.copyWith(notesEdited: 9)),
        ReviewIneligibility.tooFewNotes,
      );
      expect(_why(_happy.copyWith(notesEdited: 10)), isNull);
    });

    test('keeps quiet for three days after trouble', () {
      expect(
        _why(_happy.copyWith(lastTroubleAt: _daysAgo(2))),
        ReviewIneligibility.recentTrouble,
      );
      expect(_why(_happy.copyWith(lastTroubleAt: _daysAgo(3))), isNull);
    });

    test('waits 120 days after asking, and never asks twice on a version', () {
      expect(
        _why(_happy.copyWith(lastAskedAt: _daysAgo(119))),
        ReviewIneligibility.askedRecently,
      );
      final asked = _happy.copyWith(
        lastAskedAt: _daysAgo(120),
        askedVersions: <String>['1.2.0'],
      );
      expect(_why(asked), ReviewIneligibility.askedThisVersion);
      expect(_why(asked, version: '1.3.0'), isNull);
    });

    test('a time after now (the clock went back) keeps the app quiet', () {
      final later = _now.add(const Duration(days: 30));
      expect(
        _why(_happy.copyWith(firstOpenedAt: later)),
        ReviewIneligibility.tooNew,
      );
      expect(
        _why(_happy.copyWith(lastAskedAt: later)),
        ReviewIneligibility.askedRecently,
      );
      expect(
        _why(_happy.copyWith(lastTroubleAt: later)),
        ReviewIneligibility.recentTrouble,
      );
    });

    Glados<ReviewPromptRecord>(
      any.reviewRecord,
      kansoExploreConfig(),
    ).test('is never eligible within 120 days of the last ask', (record) {
      final asked = record.lastAskedAt;
      if (asked != null && _now.difference(asked) < reviewCooldown) {
        expect(_why(record), isNotNull);
      }
    });

    Glados<ReviewPromptRecord>(
      any.reviewRecord,
      kansoExploreConfig(),
    ).test('is never eligible on a version that already asked', (record) {
      for (final version in record.askedVersions) {
        expect(_why(record, version: version), isNotNull);
      }
    });

    Glados<ReviewPromptRecord>(
      any.reviewRecord,
      kansoExploreConfig(),
    ).test('is never eligible with recent trouble', (record) {
      final trouble = record.lastTroubleAt;
      if (trouble != null &&
          _now.difference(trouble) < reviewQuietAfterTrouble) {
        expect(_why(record), isNotNull);
      }
    });

    Glados<ReviewPromptRecord>(
      any.reviewRecord,
      kansoExploreConfig(),
    ).test('once asked, is not eligible again at once', (record) {
      const version = '9.9.9';
      final asked = reviewAsked(record, now: _now, version: version);
      expect(_why(asked, version: version), isNotNull);
      expect(
        reviewEligible(
          asked,
          now: _now.add(reviewCooldown - const Duration(minutes: 1)),
          version: '10.0.0',
        ),
        isFalse,
      );
    });

    Glados<ReviewPromptRecord>(
      any.reviewRecord,
      kansoExploreConfig(),
    ).test('eligible means every rule holds', (record) {
      if (_why(record) != null) {
        return;
      }
      final first = record.firstOpenedAt!;
      expect(_now.difference(first), greaterThanOrEqualTo(reviewMinimumAge));
      expect(record.activeDays, greaterThanOrEqualTo(reviewMinimumActiveDays));
      expect(record.notesEdited, greaterThanOrEqualTo(reviewMinimumNotes));
      expect(record.askedVersions, isNot(contains('1.2.0')));
    });
  });

  group('reviewOpened', () {
    test('keeps the first open and counts each day once', () {
      final morning = DateTime(2026, 11, 2, 8);
      var record = reviewOpened(const ReviewPromptRecord(), morning);
      expect(record.firstOpenedAt, morning);
      expect(record.activeDays, 1);
      record = reviewOpened(record, DateTime(2026, 11, 2, 21));
      expect(record.activeDays, 1);
      expect(record.firstOpenedAt, morning);
      record = reviewOpened(record, DateTime(2026, 11, 3, 7));
      expect(record.activeDays, 2);
      expect(record.lastActiveDay, '2026-11-03');
    });
  });

  group('reviewAsked', () {
    test('keeps the time and the version, the last ten at most', () {
      var record = _happy;
      for (var minor = 0; minor < 12; minor++) {
        record = reviewAsked(record, now: _now, version: '1.$minor.0');
      }
      expect(record.lastAskedAt, _now);
      expect(record.askedVersions, hasLength(reviewAskedVersionsKept));
      expect(record.askedVersions.last, '1.11.0');
      expect(record.askedVersions.first, '1.2.0');
      record = reviewAsked(record, now: _now, version: '1.5.0');
      expect(record.askedVersions.last, '1.5.0');
      expect(record.askedVersions.where((v) => v == '1.5.0'), hasLength(1));
    });
  });

  group('isReviewListLocation', () {
    test('is a list with no note open', () {
      for (final path in <String>[
        '/notes',
        '/pinned',
        '/recent',
        '/tasks',
        '/notebooks/abc',
        '/tags/t1',
      ]) {
        expect(isReviewListLocation(path), isTrue, reason: path);
      }
    });

    test('is not a note, a board, settings or anything else', () {
      for (final path in <String>[
        '/',
        '',
        '/notes/note/n1',
        '/notebooks/abc/note/n1',
        '/notebooks/abc/board',
        '/search',
        '/settings',
        '/settings/about',
        '/trash',
        '/new',
        '/ask',
        '/triage',
      ]) {
        expect(isReviewListLocation(path), isFalse, reason: path);
      }
    });
  });
}
