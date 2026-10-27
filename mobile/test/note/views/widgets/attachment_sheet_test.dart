// Kanso — attachment_sheet_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' as glados;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/note/views/widgets/attachment_sheet.dart';

import '../../../helpers/golden.dart';

final AttachmentFile _pdf = AttachmentFile(
  id: 'file:plan',
  name: 'Garden plan 2026.pdf',
  contentType: 'application/pdf',
  bytes: Uint8List(248300),
);

void main() {
  group('byteSize', () {
    test('reads as a person reads a size', () {
      expect(byteSize(0), '0 B');
      expect(byteSize(999), '999 B');
      expect(byteSize(1000), '1.0 KB');
      expect(byteSize(3400), '3.4 KB');
      expect(byteSize(248300), '248 KB');
      expect(byteSize(12000000), '12 MB');
      expect(byteSize(5000000000000), '5000 GB');
    });

    // Property: a larger file never reads as a smaller unit.
    glados.Glados2<int, int>(
      glados.any.intInRange(0, 1 << 40),
      glados.any.intInRange(0, 1 << 40),
    ).test('the unit never goes down as the size goes up', (a, b) {
      const units = <String>['B', 'KB', 'MB', 'GB'];
      final (small, large) = a <= b ? (a, b) : (b, a);
      int unitOf(int n) => units.indexOf(byteSize(n).split(' ').last);
      expect(unitOf(small), lessThanOrEqualTo(unitOf(large)));
    });
  });

  for (final brightness in Brightness.values) {
    testWidgets("an attachment's sheet, ${brightness.name}", (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        Scaffold(
          body: Align(
            alignment: Alignment.bottomCenter,
            child: Material(child: AttachmentSheet(file: _pdf)),
          ),
        ),
        name: 'phone_attachment_sheet_${brightness.name}',
        size: phoneSize,
        brightness: brightness,
      );
    });
  }
}
