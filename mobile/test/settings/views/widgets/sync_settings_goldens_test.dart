// Kanso — sync_settings_goldens_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/state/account/account_bloc.dart';

import '../../../helpers/account_blocs.dart';
import '../../../helpers/app_golden.dart';
import '../../../helpers/fixture_sync_repository.dart';
import '../../../helpers/kanso_viewport.dart';
import '../../../helpers/library_fixtures.dart';
import '../../../helpers/pump_browse_app.dart';

/// A phone signed in to an account that synced five minutes before the
/// mock content's now, with the settings given.
FixtureSyncRepository _synced({bool enabled = true}) => FixtureSyncRepository(
  current: SyncStatus(
    phase: SyncPhase.synced,
    username: 'ana',
    server: 'https://sync.kansoapp.app',
    lastSyncedMs: mockNow
        .subtract(const Duration(minutes: 5))
        .millisecondsSinceEpoch,
    conflicts: 0,
    skippedCount: 0,
    account: const SyncAccount.signedIn(),
  ),
)..syncSettings = SyncSettingsRow(enabled: enabled, intervalSecs: 900);

AccountBlocs _signedIn() => AccountBlocs(
  account: const AccountBlocState(
    account: AccountState.signedIn(
      username: 'ana',
      server: 'https://sync.kansoapp.app',
    ),
  ),
);

Future<void> _toSync(WidgetTester tester, BrowseApp app) async {
  await tester.scrollUntilVisible(
    find.text('Sync every'),
    200,
    scrollable: find.byType(Scrollable).last,
  );
  // The whole section, from its status line.
  await tester.ensureVisible(find.text('Sync now'));
  await tester.drag(find.byType(Scrollable).last, const Offset(0, 120));
  await tester.pumpAndSettle();
}

void main() {
  final viewports = <KansoViewport>[
    KansoViewport.phone,
    ...KansoViewport.goldenSet,
  ];
  final screens =
      <
        (
          String,
          FixtureSyncRepository Function(),
          Future<void> Function(WidgetTester, BrowseApp),
        )
      >[
        ('settings_sync', _synced, _toSync),
        ('settings_sync_off', () => _synced(enabled: false), _toSync),
        (
          'settings_sync_every',
          _synced,
          (tester, app) async {
            await _toSync(tester, app);
            await tester.tap(find.text('Sync every'));
            await tester.pumpAndSettle();
          },
        ),
      ];
  for (final viewport in viewports) {
    for (final brightness in Brightness.values) {
      group('${viewport.slug}, ${brightness.name}', () {
        for (final (name, sync, interact) in screens) {
          testWidgets(name, (tester) async {
            await expectAppGoldenOnBothPlatforms(
              tester,
              name: '${viewport.slug}_${name}_${brightness.name}',
              viewport: viewport,
              brightness: brightness,
              at: '/settings',
              accounts: _signedIn,
              sync: sync,
              interact: interact,
            );
          });
        }
      });
    }
  }
}
