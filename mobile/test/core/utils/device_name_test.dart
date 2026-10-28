// Kanso — device_name_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados, IntAnys, any;
import 'package:kanso/core/configs/kanso_layout.dart';
import 'package:kanso/core/utils/device_name.dart';

import '../../helpers/property.dart';

void main() {
  group('deviceNameFor', () {
    test('names the phone and the tablet of each platform', () {
      expect(deviceNameFor(TargetPlatform.iOS, 393), 'iPhone');
      expect(deviceNameFor(TargetPlatform.iOS, 820), 'iPad');
      expect(deviceNameFor(TargetPlatform.android, 411), 'Android phone');
      expect(deviceNameFor(TargetPlatform.android, 800), 'Android tablet');
    });

    test('a tablet starts at the shortest side the layout calls one', () {
      const edge = KansoLayout.tabletShortestSide;
      expect(deviceNameFor(TargetPlatform.iOS, edge - 1), 'iPhone');
      expect(deviceNameFor(TargetPlatform.iOS, edge), 'iPad');
    });

    Glados<int>(any.intInRange(200, 2000), kansoExploreConfig()).test(
      'is never empty, and is a tablet’s from the same width on every '
      'platform',
      (side) {
        final ios = deviceNameFor(TargetPlatform.iOS, side.toDouble());
        final android = deviceNameFor(TargetPlatform.android, side.toDouble());
        expect(ios, isNotEmpty);
        expect(android, isNotEmpty);
        expect(
          ios == 'iPad',
          android == 'Android tablet',
          reason: 'one width is a tablet on both platforms or neither',
        );
      },
    );
  });
}
