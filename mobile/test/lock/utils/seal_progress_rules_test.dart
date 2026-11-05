// Kanso — seal_progress_rules_test.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados2, Glados3, IntAnys, any;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/lock/utils/seal_progress_rules.dart';

import '../../helpers/property.dart';

SealProgress _progress(int done, int total, {int bytes = 0}) => SealProgress(
  phase: SealPhase.sealing,
  done: done,
  total: total,
  bytesDone: 0,
  bytesTotal: bytes,
);

void main() {
  group('wantsProgressSheet', () {
    test('four files or eight MiB are long enough, less is not', () {
      expect(wantsProgressSheet(files: 3, bytes: sheetBytes - 1), isFalse);
      expect(wantsProgressSheet(files: sheetFiles, bytes: 0), isTrue);
      expect(wantsProgressSheet(files: 1, bytes: sheetBytes), isTrue);
    });

    Glados2<int, int>(any.int, any.int, kansoExploreConfig()).test(
      'it is exactly the two thresholds, as the desktop draws the line',
      (files, bytes) {
        expect(
          wantsProgressSheet(files: files, bytes: bytes),
          files >= 4 || bytes >= 8 * 1024 * 1024,
        );
      },
    );

    Glados2<int, int>(any.int, any.int, kansoExploreConfig()).test(
      'progress asks for the sheet as its totals do',
      (files, bytes) {
        expect(
          progressWantsSheet(_progress(0, files, bytes: bytes)),
          wantsProgressSheet(files: files, bytes: bytes),
        );
      },
    );
  });

  group('sealFraction', () {
    test('nothing to do is no progress, all done is full', () {
      expect(sealFraction(_progress(0, 0)), 0);
      expect(sealFraction(_progress(12, 12)), 1);
      expect(sealFraction(_progress(5, 10)), 0.5);
    });

    Glados3<int, int, int>(
      any.int,
      any.int,
      any.int,
      kansoExploreConfig(),
    ).test(
      'it stays between 0 and 1 and never goes back as files finish',
      (done, total, more) {
        final before = sealFraction(_progress(done, total));
        final after = sealFraction(_progress(done + more.abs(), total));
        expect(before, inInclusiveRange(0, 1));
        expect(after, inInclusiveRange(0, 1));
        if (total > 0 && done >= 0) {
          expect(after, greaterThanOrEqualTo(before));
        }
      },
    );
  });
}
