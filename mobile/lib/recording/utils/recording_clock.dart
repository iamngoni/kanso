// Kanso — recording_clock.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// `00:42:17`: the recording bar's clock, audio time. The same text the
/// desktop's bar shows.
String recordingClock(int seconds) {
  final total = seconds < 0 ? 0 : seconds;
  String two(int n) => n.toString().padLeft(2, '0');
  return '${two(total ~/ 3600)}:${two(total ~/ 60 % 60)}:${two(total % 60)}';
}
