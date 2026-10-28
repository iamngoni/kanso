// Kanso — long_note_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';

import '../../integration_test/helpers/long_note.dart';

void main() {
  group('longNote', () {
    test('has at least the words asked for, and not many more', () {
      for (final words in <int>[100, 5000, 50000]) {
        final count = longNote(words).trim().split(RegExp(r'\s+')).length;
        expect(count, greaterThanOrEqualTo(words));
        expect(count, lessThan(words + 80));
      }
    });

    test('is the same every time', () {
      expect(longNote(3000), longNote(3000));
      expect(fixtureHash(fiftyThousandWordNote()), hasLength(16));
    });

    test('is a prefix-stable mix of what notes hold', () {
      final note = longNote(2000);
      for (final part in <String>[
        '## Section',
        '- [ ] task',
        '| a ',
        '```rust',
        '[[Note',
        '日本語',
        '🙂',
        '> quoted',
        'after the broken table',
      ]) {
        expect(note, contains(part));
      }
    });
  });

  group('fixtureHash', () {
    test('is FNV-1a, 64 bits', () {
      // The reference vectors of the algorithm.
      expect(fixtureHash(''), 'cbf29ce484222325');
      expect(fixtureHash('a'), 'af63dc4c8601ec8c');
      expect(fixtureHash('foobar'), '85944171f73967e8');
    });

    test('tells texts apart', () {
      expect(fixtureHash('a'), isNot(fixtureHash('b')));
    });
  });
}
