// Kanso — history_replace_backups_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/router/kanso_routes.dart';

import '../helpers/app_golden.dart';
import '../helpers/fixture_parity_repositories.dart';
import '../helpers/kanso_viewport.dart';
import '../helpers/library_fixtures.dart';
import '../helpers/pump_browse_app.dart';

/// The five phone and tablet shapes, in both tablet orientations.
const List<KansoViewport> _viewports = <KansoViewport>[
  KansoViewport.phone,
  KansoViewport.iPadPortrait,
  KansoViewport.iPadLandscape,
  KansoViewport.androidTablet,
  KansoViewport.androidTabletPortrait,
];

void main() {
  for (final viewport in _viewports) {
    for (final brightness in Brightness.values) {
      testWidgets('history, ${viewport.slug} ${brightness.name}', (
        tester,
      ) async {
        await expectAppGoldenOnBothPlatforms(
          tester,
          name: 'history_${viewport.slug}_${brightness.name}',
          viewport: viewport,
          brightness: brightness,
          at: '${KansoRoutes.history}/${MockIds.syncDesign}',
          history: FixtureHistoryRepository.new,
        );
      });

      testWidgets('replace, ${viewport.slug} ${brightness.name}', (
        tester,
      ) async {
        await expectAppGoldenOnBothPlatforms(
          tester,
          name: 'replace_${viewport.slug}_${brightness.name}',
          viewport: viewport,
          brightness: brightness,
          at: KansoRoutes.replace,
          replace: FixtureReplaceRepository.new,
        );
      });

      testWidgets('backups, ${viewport.slug} ${brightness.name}', (
        tester,
      ) async {
        await expectAppGoldenOnBothPlatforms(
          tester,
          name: 'backups_${viewport.slug}_${brightness.name}',
          viewport: viewport,
          brightness: brightness,
          at: KansoRoutes.settingsBackups,
          backups: FixtureBackupsRepository.new,
        );
      });
    }
  }

  testWidgets('history restores the selected version', (tester) async {
    final history = FixtureHistoryRepository();
    await pumpBrowseApp(
      tester,
      KansoViewport.phone.size,
      at: '${KansoRoutes.history}/${MockIds.syncDesign}',
      history: history,
    );
    expect(find.textContaining('An earlier line.'), findsOneWidget);
    await tester.tap(find.text('Restore this version'));
    await tester.pumpAndSettle();
    expect(history.restores, <(String, String)>[
      (MockIds.syncDesign, 'rev-old'),
    ]);
  });

  testWidgets('replace applies once and the same batch can be undone', (
    tester,
  ) async {
    final replace = FixtureReplaceRepository();
    await pumpBrowseApp(
      tester,
      KansoViewport.phone.size,
      at: KansoRoutes.replace,
      replace: replace,
    );
    await tester.enterText(find.byType(TextField).first, 'needle');
    await tester.pumpAndSettle();
    expect(find.text('Field notes'), findsOneWidget);
    await tester.tap(find.text('Replace 2 in 1 note'));
    await tester.pumpAndSettle();
    expect(replace.applied, hasLength(1));
    expect(find.text('Replaced 2 matches in 1 note.'), findsOneWidget);
    await tester.tap(find.text('Undo replace'));
    await tester.pumpAndSettle();
    expect(replace.undone, <String>['batch-1']);
    expect(find.text('Put back 1 note.'), findsOneWidget);
  });

  testWidgets('Back up now makes a manual copy that can be deleted', (
    tester,
  ) async {
    final backups = FixtureBackupsRepository();
    await pumpBrowseApp(
      tester,
      KansoViewport.phone.size,
      at: KansoRoutes.settingsBackups,
      backups: backups,
    );
    expect(backups.dailyChecks, 1);
    await tester.tap(find.text('Back up now'));
    await tester.pumpAndSettle();
    expect(backups.manualCount, 1);
    await tester.tap(find.byTooltip('Delete backup'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Delete').last);
    await tester.pumpAndSettle();
    expect(backups.files, hasLength(1));
  });
}
