// Kanso — seal_size_test.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados, IntAnys, any;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/gen/l10n/app_localizations_en.dart';
import 'package:kanso/lock/utils/copy_notice_words.dart';
import 'package:kanso/lock/utils/seal_size.dart';

import '../../helpers/property.dart';

const String _small = 'under 1 KB';

void main() {
  group('sealSize', () {
    test('says sizes in units of 1024, as the desktop does', () {
      expect(sealSize(0, underOneKb: _small), _small);
      expect(sealSize(1023, underOneKb: _small), _small);
      expect(sealSize(1024, underOneKb: _small), '1 KB');
      expect(sealSize(512 * 1024, underOneKb: _small), '512 KB');
      expect(sealSize(1024 * 1024, underOneKb: _small), '1.0 MB');
      expect(sealSize(3 * 1024 * 1024, underOneKb: _small), '3.0 MB');
      expect(sealSize(10 * 1024 * 1024, underOneKb: _small), '10 MB');
      expect(sealSize(84 * 1024 * 1024, underOneKb: _small), '84 MB');
    });

    Glados<int>(any.int, kansoExploreConfig()).test(
      'a size is never empty and names a unit',
      (raw) {
        final bytes = raw.abs();
        final words = sealSize(bytes, underOneKb: _small);
        expect(
          words == _small || words.endsWith(' KB') || words.endsWith(' MB'),
          isTrue,
          reason: words,
        );
        expect(words == _small, bytes < 1024);
      },
    );
  });

  group('copyNoticeWords', () {
    final l10n = AppLocalizationsEn();

    test('says nothing when nothing was copied', () {
      expect(copyNoticeWords(l10n, null), isNull);
      expect(
        copyNoticeWords(l10n, const LockCopyNotice(removed: true, copied: 0)),
        isNull,
      );
    });

    test("says the desktop's words after a lock", () {
      expect(
        copyNoticeWords(l10n, const LockCopyNotice(removed: false, copied: 1)),
        'Locked. 1 attachment is also used by other notes, so this note has '
        'its own sealed copy.',
      );
      expect(
        copyNoticeWords(l10n, const LockCopyNotice(removed: false, copied: 3)),
        'Locked. 3 attachments are also used by other notes, so this note '
        'has its own sealed copies.',
      );
    });

    test("says the desktop's words after Remove Lock", () {
      expect(
        copyNoticeWords(l10n, const LockCopyNotice(removed: true, copied: 1)),
        'Lock removed. 1 attachment was copied for this note, because '
        'another locked note still shows the sealed one.',
      );
      expect(
        copyNoticeWords(l10n, const LockCopyNotice(removed: true, copied: 2)),
        'Lock removed. 2 attachments were copied for this note, because '
        'another locked note still shows the sealed ones.',
      );
    });
  });
}
