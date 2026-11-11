// Kanso — clock_time.dart
// Created 2026-11-09.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// The `HH:mm` text the core stores for a time of day, such as the daily
/// flashcards reminder.
String formatClockTime(int hour, int minute) {
  final hh = hour.toString().padLeft(2, '0');
  final mm = minute.toString().padLeft(2, '0');
  return '$hh:$mm';
}
