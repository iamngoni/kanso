// Kanso — review_eligibility.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../models/data/review_prompt_record.dart';

/// How long after the first open the app may ask for a review.
const Duration reviewMinimumAge = Duration(days: 7);

/// On how many different days the app must have been opened.
const int reviewMinimumActiveDays = 5;

/// How many edit sessions must have been written in (a note opened,
/// typed in and let go counts once each time, not once per note).
const int reviewMinimumNotes = 10;

/// How long after a crash or a sync or sign-in failure the app keeps quiet.
const Duration reviewQuietAfterTrouble = Duration(days: 3);

/// The least time between two requests from the app (the system caps its
/// own on top of this).
const Duration reviewCooldown = Duration(days: 120);

/// How many asked versions [ReviewPromptRecord.askedVersions] keeps.
const int reviewAskedVersionsKept = 10;

/// Why the app may not ask for a review now.
enum ReviewIneligibility {
  /// Fewer than [reviewMinimumAge] since the first open.
  tooNew,

  /// Opened on fewer than [reviewMinimumActiveDays] days.
  tooFewDays,

  /// Fewer than [reviewMinimumNotes] notes written in.
  tooFewNotes,

  /// A crash or a sync or sign-in failure within [reviewQuietAfterTrouble].
  recentTrouble,

  /// Asked within [reviewCooldown].
  askedRecently,

  /// Already asked from this version of the app.
  askedThisVersion,
}

/// Why the app may not ask for a review at [now] from [version], given what
/// [record] counted; null when it may. A time in [record] that is after
/// [now] (the clock went back) counts as just now, so it keeps the app
/// quiet rather than letting it ask early.
ReviewIneligibility? reviewIneligibility(
  ReviewPromptRecord record, {
  required DateTime now,
  required String version,
}) {
  bool within(DateTime? at, Duration span) =>
      at != null && now.difference(at) < span;

  final firstOpenedAt = record.firstOpenedAt;
  if (firstOpenedAt == null || within(firstOpenedAt, reviewMinimumAge)) {
    return ReviewIneligibility.tooNew;
  }
  if (record.activeDays < reviewMinimumActiveDays) {
    return ReviewIneligibility.tooFewDays;
  }
  if (record.notesEdited < reviewMinimumNotes) {
    return ReviewIneligibility.tooFewNotes;
  }
  if (within(record.lastTroubleAt, reviewQuietAfterTrouble)) {
    return ReviewIneligibility.recentTrouble;
  }
  if (within(record.lastAskedAt, reviewCooldown)) {
    return ReviewIneligibility.askedRecently;
  }
  if (record.askedVersions.contains(version)) {
    return ReviewIneligibility.askedThisVersion;
  }
  return null;
}

/// Whether the app may ask for a review: see [reviewIneligibility].
bool reviewEligible(
  ReviewPromptRecord record, {
  required DateTime now,
  required String version,
}) => reviewIneligibility(record, now: now, version: version) == null;

/// [record] after the app was opened at [now]: the first open kept, and a
/// day it was not yet opened on counted.
ReviewPromptRecord reviewOpened(ReviewPromptRecord record, DateTime now) {
  final day = reviewDay(now);
  return record.copyWith(
    firstOpenedAt: record.firstOpenedAt ?? now.toUtc(),
    activeDays: record.lastActiveDay == day
        ? record.activeDays
        : record.activeDays + 1,
    lastActiveDay: day,
  );
}

/// [record] after the app asked for a review at [now] from [version].
ReviewPromptRecord reviewAsked(
  ReviewPromptRecord record, {
  required DateTime now,
  required String version,
}) {
  final versions = <String>[
    for (final asked in record.askedVersions)
      if (asked != version) asked,
    version,
  ];
  final start = versions.length > reviewAskedVersionsKept
      ? versions.length - reviewAskedVersionsKept
      : 0;
  return record.copyWith(
    lastAskedAt: now.toUtc(),
    askedVersions: versions.sublist(start),
  );
}

/// The local day of [at], `YYYY-MM-DD`.
String reviewDay(DateTime at) {
  final local = at.toLocal();
  return '${local.year.toString().padLeft(4, '0')}-'
      '${local.month.toString().padLeft(2, '0')}-'
      '${local.day.toString().padLeft(2, '0')}';
}

/// Whether [path] (a location's path, without its query) is a list of
/// notes or tasks with no note open: where a review may be asked for.
bool isReviewListLocation(String path) {
  final segments = Uri.parse(
    path,
  ).pathSegments.where((segment) => segment.isNotEmpty).toList();
  if (segments.isEmpty) {
    return false;
  }
  if (segments.contains('note') || segments.contains('board')) {
    return false;
  }
  return switch (segments.first) {
    'notes' || 'pinned' || 'recent' || 'tasks' => segments.length == 1,
    'notebooks' || 'tags' => segments.length <= 2,
    _ => false,
  };
}
