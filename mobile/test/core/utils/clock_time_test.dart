// Kanso — clock_time_test.dart
// Created 2026-11-09.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math';

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/utils/clock_time.dart';

void main() {
  test('every time of day formats as a two-digit HH:mm that parses back', () {
    final random = Random(360);
    for (var i = 0; i < 500; i++) {
      final hour = random.nextInt(24);
      final minute = random.nextInt(60);
      final text = formatClockTime(hour, minute);
      expect(RegExp(r'^\d{2}:\d{2}$').hasMatch(text), isTrue, reason: text);
      final parts = text.split(':');
      expect(int.parse(parts[0]), hour);
      expect(int.parse(parts[1]), minute);
    }
  });

  test('single digits are padded', () {
    expect(formatClockTime(7, 5), '07:05');
    expect(formatClockTime(0, 0), '00:00');
    expect(formatClockTime(23, 59), '23:59');
  });
}
