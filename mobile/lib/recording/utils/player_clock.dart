// Kanso — player_clock.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Time in a recording as the card writes it: `12:34`, or `1:02:03` past an
/// hour. Negative times are zero. The same words as a timestamp link's
/// text in the note.
String playerClock(int ms) {
  final total = ms < 0 ? 0 : ms ~/ 1000;
  final h = total ~/ 3600;
  final m = total ~/ 60 % 60;
  final s = total % 60;
  String two(int n) => n.toString().padLeft(2, '0');
  return h > 0 ? '$h:${two(m)}:${two(s)}' : '${two(m)}:${two(s)}';
}

/// How far along a recording is, 0 to 1. Zero for one with no length.
double playerProgress(int positionMs, int lengthMs) {
  if (lengthMs <= 0) {
    return 0;
  }
  return (positionMs / lengthMs).clamp(0.0, 1.0);
}

/// The moment [fraction] of the way along a recording, in milliseconds,
/// held within it.
int playerMomentAt(double fraction, int lengthMs) {
  if (lengthMs <= 0 || fraction.isNaN) {
    return 0;
  }
  return (lengthMs * fraction.clamp(0.0, 1.0)).round().clamp(0, lengthMs);
}

/// `1×`, `0.75×`, `1.25×`.
String playerSpeedLabel(double speed) {
  var text = speed.toStringAsFixed(2);
  if (text.contains('.')) {
    text = text
        .replaceFirst(RegExp(r'0+$'), '')
        .replaceFirst(RegExp(r'\.$'), '');
  }
  return '$text×';
}
