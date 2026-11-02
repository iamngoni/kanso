// Kanso — review_prompt_test.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:clock/clock.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/review_moment.dart';
import 'package:kanso/core/models/data/review_prompt_record.dart';
import 'package:kanso/core/services/review_prompt.dart';
import 'package:kanso/crash/models/data/pending_crash_report.dart';

import '../../helpers/crash_samples.dart';
import '../../helpers/fixture_about_repository.dart';
import '../../helpers/review_prompt_fixtures.dart';

final DateTime _now = DateTime.utc(2026, 11, 2, 12);

/// Someone who qualifies: a month of use, on many days, with many notes.
final ReviewPromptRecord _happy = ReviewPromptRecord(
  firstOpenedAt: _now.subtract(const Duration(days: 30)),
  activeDays: 12,
  notesEdited: 40,
);

class _Setup {
  _Setup({ReviewPromptRecord? record, List<String> crashDays = const []})
    : records = MemoryReviewPromptRepository(record ?? _happy),
      crashes = FixtureCrashReportRepository(<PendingCrashReport>[
        for (final (i, day) in crashDays.indexed)
          samplePending('c$i', sampleCrashReport(day: day)),
      ]) {
    prompt = ReviewPrompt(records, store, FixtureAboutRepository(), crashes);
  }

  final MemoryReviewPromptRepository records;
  final FixtureStoreReviewRepository store = FixtureStoreReviewRepository();
  final FixtureCrashReportRepository crashes;
  late final ReviewPrompt prompt;
}

Future<T> _at<T>(DateTime at, Future<T> Function() body) =>
    withClock(Clock.fixed(at), body);

void main() {
  group('ReviewPrompt', () {
    test('asks once after a good moment, and remembers it', () async {
      final setup = _Setup();
      final asked = await _at(_now, () async {
        setup.prompt.moment(ReviewMoment.taskDone);
        return setup.prompt.askIfDue();
      });
      expect(asked, isTrue);
      expect(setup.store.requests, 1);
      expect(setup.records.record.lastAskedAt, _now);
      // FixtureAboutRepository's version.
      expect(setup.records.record.askedVersions, <String>['0.1.0']);

      final again = await _at(_now.add(const Duration(minutes: 1)), () async {
        setup.prompt.moment(ReviewMoment.published);
        return setup.prompt.askIfDue();
      });
      expect(again, isFalse);
      expect(setup.store.requests, 1);
    });

    test('never asks without a good moment, or after it went stale', () async {
      final setup = _Setup();
      expect(await _at(_now, setup.prompt.askIfDue), isFalse);
      await _at(_now, () async {
        setup.prompt.moment(ReviewMoment.noteClosedAfterEdit);
      });
      final late = _now.add(reviewMomentFreshness);
      expect(await _at(late, setup.prompt.askIfDue), isFalse);
      expect(setup.store.requests, 0);
    });

    test('a moment is used up even when the person does not qualify', () async {
      final setup = _Setup(record: const ReviewPromptRecord());
      await _at(_now, () async {
        setup.prompt.moment(ReviewMoment.taskDone);
        expect(setup.prompt.hasPendingMoment, isTrue);
        expect(await setup.prompt.askIfDue(), isFalse);
        expect(setup.prompt.hasPendingMoment, isFalse);
      });
      expect(setup.store.requests, 0);
    });

    test('a fresh install is never asked in its first week', () async {
      final setup = _Setup(record: const ReviewPromptRecord());
      for (var day = 0; day < 7; day++) {
        final at = _now.add(Duration(days: day));
        await _at(at, () async {
          await setup.prompt.opened();
          for (var i = 0; i < 3; i++) {
            await setup.prompt.noteEdited();
          }
          expect(await setup.prompt.askIfDue(), isFalse, reason: 'day $day');
        });
      }
      expect(setup.records.record.activeDays, 7);
      expect(setup.records.record.notesEdited, 21);
      expect(setup.store.requests, 0);
      // The eighth day it may.
      final asked = await _at(_now.add(const Duration(days: 7)), () async {
        await setup.prompt.noteEdited();
        return setup.prompt.askIfDue();
      });
      expect(asked, isTrue);
    });

    test('keeps quiet after a crash in the last three days', () async {
      final setup = _Setup(crashDays: <String>['2026-11-01']);
      final asked = await _at(_now, () async {
        setup.prompt.moment(ReviewMoment.taskDone);
        return setup.prompt.askIfDue();
      });
      expect(asked, isFalse);
      expect(setup.store.requests, 0);
      expect(setup.records.record.lastTroubleAt, _now);
    });

    test('an old crash report does not keep it quiet', () async {
      final setup = _Setup(crashDays: <String>['2026-10-20']);
      final asked = await _at(_now, () async {
        setup.prompt.moment(ReviewMoment.taskDone);
        return setup.prompt.askIfDue();
      });
      expect(asked, isTrue);
    });

    test('keeps quiet after sync or sign-in trouble', () async {
      final setup = _Setup();
      await _at(_now, setup.prompt.trouble);
      final asked = await _at(_now.add(const Duration(days: 2)), () async {
        setup.prompt.moment(ReviewMoment.firstSync);
        return setup.prompt.askIfDue();
      });
      expect(asked, isFalse);
    });

    test('does not ask where the store cannot', () async {
      final setup = _Setup();
      setup.store.isAvailable = false;
      final asked = await _at(_now, () async {
        setup.prompt.moment(ReviewMoment.taskDone);
        return setup.prompt.askIfDue();
      });
      expect(asked, isFalse);
      expect(setup.records.record.lastAskedAt, isNull);
    });

    test('does not ask when it could not remember asking', () async {
      final setup = _Setup();
      setup.records.failing = true;
      final asked = await _at(_now, () async {
        setup.prompt.moment(ReviewMoment.taskDone);
        return setup.prompt.askIfDue();
      });
      expect(asked, isFalse);
      expect(setup.store.requests, 0);
    });

    test('only the first sync is a good moment', () async {
      final setup = _Setup();
      await _at(_now, () async {
        await setup.prompt.synced();
        expect(setup.prompt.hasPendingMoment, isTrue);
        expect(setup.records.record.firstSyncSeen, isTrue);
        await setup.prompt.askIfDue();
        await setup.prompt.synced();
        expect(setup.prompt.hasPendingMoment, isFalse);
      });
    });
  });
}
