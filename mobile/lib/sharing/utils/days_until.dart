// Kanso — days_until.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// One day, in milliseconds.
const int millisecondsPerDay = 86400000;

/// Whole days from [nowMs] until [atMs], rounded up; 0 once [atMs] has come.
///
/// An invite link that has 1 hour left has 1 day left, as on the desktop.
int daysUntil(int atMs, int nowMs) {
  final left = atMs - nowMs;
  if (left <= 0) {
    return 0;
  }
  return (left + millisecondsPerDay - 1) ~/ millisecondsPerDay;
}
