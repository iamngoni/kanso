// Kanso — capture_sheet_goldens_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../../../helpers/app_golden.dart';
import '../../../helpers/kanso_viewport.dart';
import '../../../helpers/pump_browse_app.dart';

/// Holding the list's new-note control offers the camera, photos and PDFs.
Future<void> _holdNewNote(WidgetTester tester, BrowseApp app) async {
  await tester.longPress(find.bySemanticsLabel('New note').first);
  await tester.pumpAndSettle();
}

/// The phone, and the three tablets the issue asks for in both orientations.
const List<KansoViewport> _everywhere = <KansoViewport>[
  KansoViewport.phone,
  KansoViewport.iPadPortrait,
  KansoViewport.iPadLandscape,
  KansoViewport.androidTablet,
  KansoViewport.androidTabletPortrait,
];

void main() {
  for (final viewport in _everywhere) {
    for (final brightness in Brightness.values) {
      testWidgets('the capture sheet, ${viewport.slug} ${brightness.name}', (
        tester,
      ) async {
        await expectAppGoldenOnBothPlatforms(
          tester,
          name: 'capture_sheet_${viewport.slug}_${brightness.name}',
          viewport: viewport,
          brightness: brightness,
          interact: _holdNewNote,
        );
      });
    }
  }
}
