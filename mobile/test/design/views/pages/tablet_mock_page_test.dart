// Kanso — tablet_mock_page_test.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/design/models/data/design_screen.dart';
import 'package:kanso/design/views/pages/tablet_mock_page.dart';

import '../../../helpers/golden.dart';

void main() {
  group('TabletMockPage', () {
    testWidgets('error, light', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const TabletMockPage(screen: DesignScreen.error),
        name: 'tablet_error_light',
        size: tabletLandscapeSize,
        brightness: Brightness.light,
      );
    });

    testWidgets('error, dark', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const TabletMockPage(screen: DesignScreen.error),
        name: 'tablet_error_dark',
        size: tabletLandscapeSize,
        brightness: Brightness.dark,
      );
    });
  });
}
