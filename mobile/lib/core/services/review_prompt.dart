// Kanso — review_prompt.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:clock/clock.dart';
import 'package:flutter/foundation.dart';
import 'package:injectable/injectable.dart';

import '../../crash/models/repos/abs/crash_report_repository.dart';
import '../../metrics/models/repos/abs/usage_metrics_repository.dart';
import '../../metrics/utils/count_usage.dart';
import '../models/data/review_moment.dart';
import '../models/data/review_prompt_record.dart';
import '../models/repos/abs/about_repository.dart';
import '../models/repos/abs/review_prompt_repository.dart';
import '../models/repos/abs/store_review_repository.dart';
import '../utils/review_eligibility.dart';

/// How long after a good moment the app may still ask, once the person is
/// back on a list.
const Duration reviewMomentFreshness = Duration(minutes: 10);

/// Asks people who are happy with Kanso Pocket to rate it, with the
/// system's own prompt and only at a good moment: after something went
/// well ([moment]), back on a list with the keyboard down (the host's
/// part, see `ReviewPromptHost`), and only when [reviewIneligibility] says
/// nothing against it. The "10 notes" it asks for are counted as edit
/// sessions: each time a note is let go after being written in.
///
/// What it counts stays on this phone ([ReviewPromptRepository]). Every
/// part is best effort: a failure never reaches what the person was doing.
@lazySingleton
class ReviewPrompt {
  ReviewPrompt(this._records, this._store, this._about, this._crashes);

  final ReviewPromptRepository _records;
  final StoreReviewRepository _store;
  final AboutRepository _about;
  final CrashReportRepository _crashes;

  DateTime? _momentAt;
  bool _asking = false;
  final ValueNotifier<int> _moments = ValueNotifier<int>(0);

  /// Changes each time a good moment is marked, for the host to look again
  /// whether it may ask now.
  ValueListenable<int> get moments => _moments;

  /// Whether a good moment was marked within [reviewMomentFreshness] and
  /// has not been used yet.
  bool get hasPendingMoment {
    final at = _momentAt;
    if (at == null) {
      return false;
    }
    final since = clock.now().difference(at);
    return !since.isNegative && since < reviewMomentFreshness;
  }

  /// The app was opened, or came back to the front.
  Future<void> opened() async {
    final now = clock.now();
    await _change((record) => reviewOpened(record, now));
  }

  /// A note was let go after it was written in: counted, and a good moment.
  Future<void> noteEdited() async {
    await _change(
      (record) => record.copyWith(notesEdited: record.notesEdited + 1),
    );
    moment(ReviewMoment.noteClosedAfterEdit);
  }

  /// A sync completed on this phone; the first is a good moment.
  Future<void> synced() async {
    var first = false;
    await _change((record) {
      first = !record.firstSyncSeen;
      return first ? record.copyWith(firstSyncSeen: true) : record;
    });
    if (first) {
      moment(ReviewMoment.firstSync);
    }
  }

  /// The app crashed, or sync or signing in failed: no asking for a while.
  Future<void> trouble() async {
    final now = clock.now();
    await _change((record) => record.copyWith(lastTroubleAt: now.toUtc()));
  }

  /// Something went well: the app may ask once the person is back on a
  /// list (the host calls [askIfDue]).
  void moment(ReviewMoment moment) {
    _momentAt = clock.now();
    _moments.value++;
  }

  /// Asks the system for a review if a good moment is pending and the
  /// person qualifies; the moment is used up either way. The host calls it
  /// only on a list, with the keyboard down. Whether it asked.
  Future<bool> askIfDue() async {
    if (_asking || !hasPendingMoment) {
      return false;
    }
    _asking = true;
    _momentAt = null;
    try {
      final record = (await _records.load()).fold<ReviewPromptRecord?>(
        (_) => null,
        (record) => record,
      );
      final version = (await _about.version()).fold<String?>(
        (_) => null,
        (version) => version.version,
      );
      if (record == null || version == null) {
        return false;
      }
      final now = clock.now();
      if (!reviewEligible(record, now: now, version: version)) {
        return false;
      }
      if (await _crashedRecently(now)) {
        await trouble();
        return false;
      }
      if (!await _store.available()) {
        return false;
      }
      // Kept before asking: if it cannot be remembered, it is not asked, so
      // the app can never ask twice by forgetting.
      final saved = await _records.update(
        (current) => reviewAsked(current, now: now, version: version),
      );
      if (saved.isLeft()) {
        return false;
      }
      await _store.request();
      countUsage(UsageFeature.reviewPrompt);
      return true;
    } on Object {
      return false;
    } finally {
      _asking = false;
    }
  }

  /// Saves [change] to the record; a failure is let go, as nothing the
  /// person does depends on it.
  Future<void> _change(
    ReviewPromptRecord Function(ReviewPromptRecord current) change,
  ) async {
    try {
      await _records.update(change);
    } on Object {
      // Counting never disturbs the app.
    }
  }

  /// Whether a crash report waiting on this phone is from the last
  /// [reviewQuietAfterTrouble]. A report gives only its UTC day, so a crash
  /// counts as at the end of it.
  Future<bool> _crashedRecently(DateTime now) async {
    final pending = await _crashes.pending();
    return pending.fold((_) => false, (reports) {
      for (final pending in reports) {
        final day = DateTime.tryParse('${pending.report.day}T00:00:00Z');
        if (day == null) {
          continue;
        }
        final end = day.add(const Duration(days: 1));
        if (now.toUtc().difference(end) < reviewQuietAfterTrouble) {
          return true;
        }
      }
      return false;
    });
  }
}
