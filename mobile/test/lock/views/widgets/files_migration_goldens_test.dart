// Kanso — files_migration_goldens_test.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/settings/views/widgets/privacy_settings_pane.dart';

import '../../../helpers/app_golden.dart';
import '../../../helpers/fixture_session_repository.dart';
import '../../../helpers/kanso_viewport.dart';
import '../../../helpers/pump_browse_app.dart';
import '../../../helpers/real_themes.dart';

/// Four locked notes whose twelve attachments (84 MB, three also used by
/// other notes) are not sealed yet; [counted] when the notes are open.
FilesMigration _waiting({bool counted = false, int stopped = 0}) =>
    FilesMigration(
      notes: 4,
      stopped: stopped,
      postponed: false,
      counted: counted,
      files: counted ? 12 : 0,
      bytes: counted ? 84000000 : 0,
      shared: counted ? 3 : 0,
    );

/// The notes are closed and the first-launch prompt is due.
FixtureSessionRepository _closed() =>
    FixtureSessionRepository()..migration = _waiting();

/// The notes are open, so the prompt counts the attachments.
FixtureSessionRepository _open() => FixtureSessionRepository(unlocked: true)
  ..migration = _waiting(counted: true)
  ..migrated = const MigrationOutcome(
    notes: 4,
    sealed: 9,
    copied: 3,
    unopened: 0,
    failed: 0,
    stopped: false,
  );

/// The session of the pass that is sealing: it waits at the seventh file.
late FixtureSessionRepository _sealing;

FixtureSessionRepository _sealingFiles() {
  final session = _open()..migrationGate = Completer<void>();
  _sealing = session;
  return session;
}

/// A pass that stopped part way on one note.
FixtureSessionRepository _stoppedPass() =>
    FixtureSessionRepository(unlocked: true)
      ..migration = _waiting(counted: true, stopped: 1);

Future<void> _tapText(WidgetTester tester, String text) async {
  await tester.tap(find.text(text).last);
  await tester.pumpAndSettle();
}

class _Screen {
  const _Screen(this.name, this.session, {this.interact});

  final String name;
  final FixtureSessionRepository Function() session;
  final Future<void> Function(WidgetTester tester, BrowseApp app)? interact;
}

final List<_Screen> _screens = <_Screen>[
  const _Screen('prompt', _closed),
  const _Screen('prompt_counted', _open),
  const _Screen('prompt_stopped', _stoppedPass),
  _Screen(
    'password',
    _closed,
    interact: (tester, app) => _tapText(tester, 'Seal now'),
  ),
  _Screen(
    'progress',
    _sealingFiles,
    interact: (tester, app) async {
      await tester.tap(find.text('Seal now').last);
      await tester.pump(const Duration(milliseconds: 50));
      _sealing.progress.add(
        const SealProgress(
          phase: SealPhase.sealing,
          done: 7,
          total: 12,
          bytesDone: 49000000,
          bytesTotal: 84000000,
        ),
      );
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 50));
    },
  ),
  _Screen(
    'done',
    _open,
    interact: (tester, app) => _tapText(tester, 'Seal now'),
  ),
  _Screen(
    'settings_row',
    _open,
    interact: (tester, app) async {
      await _tapText(tester, 'Later');
      unawaited(app.router.push<void>('/settings/privacy'));
      await tester.pumpAndSettle();
      await tester.scrollUntilVisible(
        find.text('Seal attachments of locked notes…'),
        200,
        scrollable: find.descendant(
          of: find.byType(PrivacySettingsPane),
          matching: find.byType(Scrollable),
        ),
      );
      await tester.pumpAndSettle();
    },
  ),
];

void main() {
  loadRealThemes();

  for (final viewport in <KansoViewport>[
    KansoViewport.phone,
    ...KansoViewport.goldenSet,
  ]) {
    for (final brightness in Brightness.values) {
      group('${viewport.slug}, ${brightness.name}', () {
        for (final screen in _screens) {
          testWidgets(screen.name, (tester) async {
            await expectAppGoldenOnBothPlatforms(
              tester,
              name:
                  'migration_${viewport.slug}_${screen.name}_${brightness.name}',
              viewport: viewport,
              brightness: brightness,
              session: screen.session,
              interact: screen.interact,
            );
          });
        }
      });
    }
  }
}
