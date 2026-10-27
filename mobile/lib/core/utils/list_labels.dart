// Kanso — list_labels.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:intl/intl.dart';

import '../../gen/l10n/app_localizations.dart';
import '../../note_list/models/data/bucket.dart';
import 'day_grouping.dart';

/// The words of a day: "Today", "Yesterday", then the date as the phone's
/// language writes it, without the year in the current year ("Oct 12") and
/// with it in any other ("Oct 12, 2025").
///
/// [day] and [today] are local days (see [localDayOf]).
String dayLabel(AppLocalizations l10n, int day, {required int today}) {
  if (day == today) {
    return l10n.dayToday;
  }
  if (day == today - 1) {
    return l10n.dayYesterday;
  }
  final date = _civilDate(day);
  final format = date.year == _civilDate(today).year
      ? DateFormat.MMMd(l10n.localeName)
      : DateFormat.yMMMd(l10n.localeName);
  return format.format(date);
}

/// What a header of the note list says: "Pinned", or the day's [dayLabel].
String bucketLabel(AppLocalizations l10n, Bucket bucket, {required int today}) {
  return switch (bucket) {
    PinnedBucket() => l10n.libraryPinned,
    DayBucket(:final localDay) => dayLabel(l10n, localDay, today: today),
  };
}

/// The time a row of the list shows for a note last at [atMs]: the hour and
/// minute when that is today, otherwise the day as a header would say it.
String rowTimeLabel(AppLocalizations l10n, int atMs, {required DateTime now}) {
  final today = localDayOf(now.millisecondsSinceEpoch);
  if (localDayOf(atMs) == today) {
    return DateFormat.Hm(
      l10n.localeName,
    ).format(DateTime.fromMillisecondsSinceEpoch(atMs));
  }
  return dayLabel(l10n, localDayOf(atMs), today: today);
}

/// A date and time in a note's facts: "Today, 09:41", "Yesterday, 18:30",
/// "Oct 21, 14:20".
String dateTimeLabel(AppLocalizations l10n, int atMs, {required DateTime now}) {
  final today = localDayOf(now.millisecondsSinceEpoch);
  final day = dayLabel(l10n, localDayOf(atMs), today: today);
  final time = DateFormat.Hm(
    l10n.localeName,
  ).format(DateTime.fromMillisecondsSinceEpoch(atMs));
  return '$day, $time';
}

/// A date in a note's facts: "Oct 12, 2026".
String dateLabel(AppLocalizations l10n, int atMs) {
  return DateFormat.yMMMd(
    l10n.localeName,
  ).format(DateTime.fromMillisecondsSinceEpoch(atMs));
}

/// The date of local day [day], as a UTC date so that no time zone moves it.
DateTime _civilDate(int day) => DateTime.utc(1970).add(Duration(days: day));
