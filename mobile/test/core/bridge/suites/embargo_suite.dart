// Kanso — embargo_suite.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';

import 'open_library.dart';

/// A note's embargo, through the bridge: `KansoLibrary.embargo`.
void embargoSuite(OpenLibrary open) {
  group('KansoLibrary.embargo', () {
    /// 18 November 2026, 09:00 in New York: 14:00 UTC.
    final lift = DateTime.utc(2026, 11, 18, 14).millisecondsSinceEpoch;

    /// Five days before it lifts.
    final now = DateTime.utc(2026, 11, 13, 14).millisecondsSinceEpoch;

    test('a zoned embargo is told with its zone until it lifts', () async {
      final library = await open();
      final id = await makeNote(
        library,
        'Pixel 11 review',
        '---\nembargo: 2026-11-18T09:00 America/New_York\n---\nWords.\n',
      );

      final view = await library.embargo(
        noteId: id,
        nowUtcMs: now,
        offsetMs: 0,
      );
      expect(view, isNotNull);
      expect(view!.lifts, 'Wed 18 Nov, 09:00 New York');
      expect(view.liftsIn, 'in 5 days');
      expect(view.liftsAtMs, lift);
      expect(view.zone, 'America/New_York');

      // Once it has lifted there is nothing to tell.
      expect(
        await library.embargo(noteId: id, nowUtcMs: lift, offsetMs: 0),
        isNull,
      );
    });

    test('a note without one, or no note at all, has none', () async {
      final library = await open();
      final id = await makeNote(library, 'Plain', 'no embargo here');
      expect(
        await library.embargo(noteId: id, nowUtcMs: now, offsetMs: 0),
        isNull,
      );
      expect(
        await library.embargo(noteId: 'note:gone', nowUtcMs: now, offsetMs: 0),
        isNull,
      );
    });
  });
}
