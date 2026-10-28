// Kanso — journal_goldens_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/layout_class.dart';
import 'package:kanso/settings/views/widgets/settings_pane.dart';

import '../../helpers/app_golden.dart';
import '../../helpers/kanso_viewport.dart';
import '../../helpers/library_fixtures.dart';
import '../../helpers/page_fixtures.dart';
import '../../helpers/pump_browse_app.dart';
import '../../helpers/real_themes.dart';

/// One screen of the journal, templates, boards and the settings that go
/// with them, as a golden of the whole window.
class _Screen {
  const _Screen(this.name, {required this.at, this.interact});

  final String name;
  final String at;
  final Future<void> Function(WidgetTester tester, BrowseApp app)? interact;
}

Future<void> _tapLabel(WidgetTester tester, String label) async {
  await tester.tap(find.bySemanticsLabel(label).first);
  await tester.pumpAndSettle();
}

Future<void> _openMove(WidgetTester tester, BrowseApp app) async {
  if (app.layoutClass.value == LayoutClass.single) {
    await tester.longPress(find.text('Weekly review'));
  } else {
    await tester.tap(find.bySemanticsLabel('Move to…').first);
  }
  await tester.pumpAndSettle();
}

Future<void> _scrollSettingsToEnd(WidgetTester tester, BrowseApp app) async {
  unawaited(app.router.push<void>('/settings'));
  await tester.pumpAndSettle();
  // Three panes keep the library and the list behind the sheet, so the
  // list to scroll is the settings pane's own.
  await tester.dragUntilVisible(
    find.text('Remind me of tasks'),
    find
        .descendant(
          of: find.byType(SettingsPane),
          matching: find.byType(ListView),
        )
        .first,
    const Offset(0, -200),
  );
  await tester.pumpAndSettle();
}

final String _daily = '/notebooks/${MockIds.journal}/note/${mockDailyNote.id}';

final List<_Screen> _screens = <_Screen>[
  _Screen('journal_note', at: Uri.encodeFull(_daily)),
  const _Screen('journal_list', at: '/notebooks/${MockIds.journal}'),
  _Screen(
    'calendar',
    at: Uri.encodeFull(_daily),
    interact: (tester, app) => _tapLabel(tester, 'Calendar'),
  ),
  _Screen(
    'new_menu',
    at: '/notes',
    interact: (tester, app) => _tapLabel(tester, 'New'),
  ),
  _Screen(
    'template_picker',
    at: '/notes',
    interact: (tester, app) async {
      await _tapLabel(tester, 'New');
      await tester.tap(find.text('New from template…'));
      await tester.pumpAndSettle();
    },
  ),
  _Screen(
    'template_new',
    at: '/notes',
    interact: (tester, app) async {
      await _tapLabel(tester, 'New');
      await tester.tap(find.text('New template'));
      await tester.pumpAndSettle();
    },
  ),
  const _Screen('board_notes', at: '/notes/board'),
  const _Screen('board_tasks', at: '/notes/board?kind=tasks'),
  const _Screen('board_move', at: '/notes/board', interact: _openMove),
  const _Screen(
    'settings_journal',
    at: '/notes',
    interact: _scrollSettingsToEnd,
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
              name: '${viewport.slug}_${screen.name}_${brightness.name}',
              viewport: viewport,
              brightness: brightness,
              at: screen.at,
              fixtures: PageFixtures.new,
              interact: screen.interact,
            );
          });
        }
      });
    }
  }
}
