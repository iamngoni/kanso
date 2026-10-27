// Kanso — time_words.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:intl/intl.dart';

import '../../gen/l10n/app_localizations.dart';
import 'day_grouping.dart';
import 'list_labels.dart';

/// When something happened, as the desktop's editor header, sync line and
/// comments say it: "just now", "5 min ago" and "3 h ago" within a day, then
/// the day as the note list names it ("yesterday", "Oct 12"). An instant in
/// the future is "just now".
String editedLabel(AppLocalizations l10n, int atMs, {required DateTime now}) {
  final nowMs = now.millisecondsSinceEpoch;
  final secs = (nowMs - atMs).clamp(0, nowMs) ~/ 1000;
  if (secs < 60) {
    return l10n.timeJustNow;
  }
  if (secs < 3600) {
    return l10n.timeMinutesAgo(secs ~/ 60);
  }
  if (secs < 86400) {
    return l10n.timeHoursAgo(secs ~/ 3600);
  }
  final day = localDayOf(atMs);
  final today = localDayOf(nowMs);
  if (day == today) {
    return l10n.timeToday;
  }
  if (day == today - 1) {
    return l10n.timeYesterday;
  }
  return dayLabel(l10n, day, today: today);
}

/// When a pause ends, to follow "Sync resumes": "at 14:05" today,
/// "tomorrow at 14:05", then "on Oct 12 at 14:05", as the desktop says it.
String resumeLabel(
  AppLocalizations l10n,
  int untilMs, {
  required DateTime now,
}) {
  final until = DateTime.fromMillisecondsSinceEpoch(untilMs);
  final time = DateFormat.Hm(l10n.localeName).format(until);
  final days = localDayOf(untilMs) - localDayOf(now.millisecondsSinceEpoch);
  if (days <= 0) {
    return l10n.timeResumeAt(time);
  }
  if (days == 1) {
    return l10n.timeResumeTomorrow(time);
  }
  return l10n.timeResumeOn(
    DateFormat.MMMd(l10n.localeName).format(until),
    time,
  );
}
