// Kanso — time_words_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados, IntAnys, any;
import 'package:intl/date_symbol_data_local.dart';
import 'package:kanso/core/utils/time_words.dart';
import 'package:kanso/gen/l10n/app_localizations_en.dart';

import '../../helpers/property.dart';

final AppLocalizationsEn _l10n = AppLocalizationsEn();

void main() {
  final now = DateTime(2026, 10, 27, 15, 30);
  final nowMs = now.millisecondsSinceEpoch;
  int ago(Duration span) => nowMs - span.inMilliseconds;

  setUpAll(() => initializeDateFormatting('en'));

  group('editedLabel', () {
    test('says it as the desktop does within a day', () {
      expect(editedLabel(_l10n, nowMs, now: now), 'just now');
      expect(
        editedLabel(_l10n, ago(const Duration(seconds: 59)), now: now),
        'just now',
      );
      expect(
        editedLabel(_l10n, ago(const Duration(minutes: 5)), now: now),
        '5 min ago',
      );
      expect(
        editedLabel(_l10n, ago(const Duration(hours: 3)), now: now),
        '3 h ago',
      );
    });

    test('then names the day', () {
      final yesterday = DateTime(2026, 10, 26, 9).millisecondsSinceEpoch;
      expect(editedLabel(_l10n, yesterday, now: now), 'yesterday');
      final earlier = DateTime(2026, 10, 12, 9).millisecondsSinceEpoch;
      expect(editedLabel(_l10n, earlier, now: now), 'Oct 12');
    });

    test('a time ahead of the clock is just now', () {
      expect(editedLabel(_l10n, nowMs + 60000, now: now), 'just now');
    });

    Glados<int>(any.intInRange(0, 86400), kansoExploreConfig()).test(
      'within a day, the count is the whole minutes or hours gone',
      (secs) {
        final label = editedLabel(
          _l10n,
          ago(Duration(seconds: secs)),
          now: now,
        );
        if (secs < 60) {
          expect(label, 'just now');
        } else if (secs < 3600) {
          expect(label, '${secs ~/ 60} min ago');
        } else {
          expect(label, '${secs ~/ 3600} h ago');
        }
      },
    );
  });

  group('resumeLabel', () {
    test('today, tomorrow, then the date', () {
      final later = DateTime(2026, 10, 27, 16, 5).millisecondsSinceEpoch;
      expect(resumeLabel(_l10n, later, now: now), 'at 16:05');
      final tomorrow = DateTime(2026, 10, 28, 8).millisecondsSinceEpoch;
      expect(resumeLabel(_l10n, tomorrow, now: now), 'tomorrow at 08:00');
      final next = DateTime(2026, 11, 2, 9, 30).millisecondsSinceEpoch;
      expect(resumeLabel(_l10n, next, now: now), 'on Nov 2 at 09:30');
    });

    Glados<int>(any.intInRange(0, 24 * 60), kansoExploreConfig()).test(
      'any time left today says only the time',
      (minutes) {
        final at = DateTime(2026, 10, 27).add(Duration(minutes: minutes));
        expect(
          resumeLabel(_l10n, at.millisecondsSinceEpoch, now: now),
          startsWith('at '),
        );
      },
    );
  });
}
