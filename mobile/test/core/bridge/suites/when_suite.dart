// Kanso — when_suite.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/bridge/api/when.dart';

import 'open_library.dart';

/// Tuesday 20 October 2026, 10:00.
const LocalTime _now = LocalTime(
  year: 2026,
  month: 10,
  day: 20,
  hour: 10,
  minute: 0,
  second: 0,
);

/// The typed-date parser, through the bridge: `whenParse` and `dateChoices`.
void whenSuite(OpenLibrary open) {
  setUpAll(() async {
    // The functions are pure, but the core is started with a library.
    await open();
  });

  group('whenParse', () {
    test('finds a date, a time and a repeat at the end of a task', () {
      final hint = whenParse(
        line: '- [ ] film Pixel review fri 3pm every week',
        now: _now,
        monthFirst: false,
      );
      expect(hint, isNotNull);
      expect(hint!.canonical, '@2026-10-23T15:00 every week');
      expect(hint.repeats, isTrue);
      expect(hint.label, 'Fri 23 Oct, 15:00 · every week');
      expect(hint.start, '- [ ] film Pixel review '.length);
      expect(hint.end, '- [ ] film Pixel review fri 3pm every week'.length);
    });

    test('says nothing of prose, a done task or a token already there', () {
      for (final line in <String>[
        'film Pixel review fri 3pm',
        '- [x] film Pixel review fri 3pm',
        '- [ ] film @2026-10-23T15:00',
      ]) {
        expect(whenParse(line: line, now: _now, monthFirst: false), isNull);
      }
    });

    test('reads 11/14 as the phone says', () {
      final dayFirst = whenParse(
        line: '- [ ] pay 14/11',
        now: _now,
        monthFirst: false,
      );
      final monthFirst = whenParse(
        line: '- [ ] pay 11/14',
        now: _now,
        monthFirst: true,
      );
      expect(dayFirst!.canonical, '@2026-11-14');
      expect(monthFirst!.canonical, '@2026-11-14');
    });
  });

  group('captureTaskHint', () {
    test('reads the date and repeat at the end of a typed task', () {
      expect(
        captureTaskHint(
          text: 'film Pixel review fri 3pm every week',
          now: _now,
          monthFirst: false,
        ),
        'Fri 23 Oct, 15:00 · every week',
      );
    });

    test('says nothing for words with no date, or a date alone', () {
      for (final text in <String>['film review', 'fri', '  ']) {
        expect(
          captureTaskHint(text: text, now: _now, monthFirst: false),
          isNull,
        );
      }
    });

    test('reads 11/14 as the phone says', () {
      expect(
        captureTaskHint(text: 'pay 14/11', now: _now, monthFirst: false),
        captureTaskHint(text: 'pay 11/14', now: _now, monthFirst: true),
      );
    });
  });

  group('dateChoices', () {
    test('lists the days after an @ in a task', () {
      final found = dateChoices(beforeCaret: '- [ ] ring @', now: _now);
      expect(found, isNotNull);
      expect(found!.start, '- [ ] ring '.length);
      expect(found.choices.first.label, 'Today');
      expect(found.choices.first.insert, '@2026-10-20');
    });

    test('offers the repeats after a date and a space', () {
      final found = dateChoices(
        beforeCaret: '- [ ] film @2026-10-23 ',
        now: _now,
      );
      expect(found!.choices.map((choice) => choice.label), <String>[
        'Every day',
        'Every week',
        'Every weekday',
        'Every month',
      ]);
    });

    test('has nothing outside a task', () {
      expect(dateChoices(beforeCaret: 'ring @', now: _now), isNull);
    });
  });
}
