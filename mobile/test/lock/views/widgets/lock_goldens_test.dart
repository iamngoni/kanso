// Kanso — lock_goldens_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/layout_class.dart';
import 'package:kanso/note_list/views/widgets/note_list_view.dart';
import 'package:kanso/settings/views/widgets/settings_pane.dart';

import '../../../helpers/app_golden.dart';
import '../../../helpers/fixture_library_repository.dart';
import '../../../helpers/fixture_session_repository.dart';
import '../../../helpers/kanso_viewport.dart';
import '../../../helpers/library_fixtures.dart';
import '../../../helpers/pump_browse_app.dart';
import '../../../helpers/real_themes.dart';

/// One screen of locked notes, as a golden of the whole window.
class _Screen {
  const _Screen(
    this.name,
    this.session, {
    this.at = '/notes',
    this.library,
    this.interact,
  });

  final String name;

  /// Makes the lock session the screen needs.
  final FixtureSessionRepository Function() session;
  final String at;
  final FixtureLibraryRepository Function()? library;
  final Future<void> Function(WidgetTester tester, BrowseApp app)? interact;
}

String _noteAt(String id) => '/notes/note/${Uri.encodeComponent(id)}';

final String _locked = _noteAt(MockIds.privateJournal);
final String _plain = _noteAt(MockIds.syncDesign);

/// The mock library with the private journal open: a locked note whose
/// notes have been unlocked.
FixtureLibraryRepository _openJournal() =>
    FixtureLibraryRepository()
      ..notes[MockIds.privateJournal] = mockNote(
        id: MockIds.privateJournal,
        title: 'Private journal',
        body: 'A quiet week. Slept well, read a lot.',
        open: true,
        notebookId: MockIds.journal,
      );

/// A password exists and the notes are closed; biometrics as [biometrics].
FixtureSessionRepository _closed({bool biometrics = false}) =>
    FixtureSessionRepository(biometrics: biometrics);

/// A password exists and the notes are open.
FixtureSessionRepository _open() =>
    FixtureSessionRepository(unlocked: true, biometrics: true);

/// No lock password yet.
FixtureSessionRepository _none() =>
    FixtureSessionRepository(hasLockPassword: false);

Future<void> _tapText(WidgetTester tester, String text) async {
  await tester.tap(find.text(text).last);
  await tester.pumpAndSettle();
}

/// Types [text] into the [index]th of the last [of] text fields on screen:
/// those of the sheet on top.
Future<void> _type(
  WidgetTester tester,
  String text, {
  int index = 0,
  int of = 1,
}) async {
  final fields = find.byType(EditableText);
  final count = fields.evaluate().length;
  await tester.enterText(fields.at(count - of + index), text);
  await tester.pump();
}

Future<void> _openMore(WidgetTester tester, BrowseApp app) async {
  await tester.tap(find.bySemanticsLabel('More').last);
  await tester.pumpAndSettle();
}

Future<void> _usePassword(WidgetTester tester, BrowseApp app) =>
    _tapText(tester, 'Use password');

Future<void> _push(WidgetTester tester, BrowseApp app, String path) async {
  unawaited(app.router.push<void>(path));
  await tester.pumpAndSettle();
}

Future<void> _settings(WidgetTester tester, BrowseApp app) async {
  await _push(tester, app, '/settings');
  await tester.scrollUntilVisible(
    find.text('Change lock password…'),
    200,
    scrollable: find.descendant(
      of: find.byType(SettingsPane),
      matching: find.byType(Scrollable),
    ),
  );
  await tester.pumpAndSettle();
}

/// Scrolls the note list to the locked note's row, as far as it goes.
Future<void> _showLockedRow(WidgetTester tester, BrowseApp app) async {
  await tester.scrollUntilVisible(
    find.text('Private journal'),
    200,
    scrollable: find
        .descendant(
          of: find.byType(NoteListView),
          matching: find.byType(Scrollable),
        )
        .first,
  );
  await tester.pumpAndSettle();
}

final List<_Screen> _screens = <_Screen>[
  const _Screen('note_list_locked', _closed, interact: _showLockedRow),
  _Screen('locked_note', _closed, at: _locked),
  _Screen(
    'locked_note_face_id',
    () => _closed(biometrics: true),
    at: _locked,
  ),
  _Screen('unlock_sheet', _closed, at: _locked, interact: _usePassword),
  _Screen(
    'unlock_sheet_face_id',
    () => _closed(biometrics: true),
    at: _locked,
    interact: _usePassword,
  ),
  _Screen(
    'unlock_sheet_wrong',
    _closed,
    at: _locked,
    interact: (tester, app) async {
      await _usePassword(tester, app);
      await _type(tester, 'not the password');
      await _tapText(tester, 'Unlock');
    },
  ),
  _Screen(
    'biometric_offer',
    _closed,
    at: _locked,
    interact: (tester, app) async {
      await _usePassword(tester, app);
      await _type(tester, 'correct horse');
      await _tapText(tester, 'Unlock');
    },
  ),
  _Screen(
    'note_menu',
    _open,
    at: _locked,
    library: _openJournal,
    interact: _openMore,
  ),
  _Screen(
    'remove_lock',
    _open,
    at: _locked,
    library: _openJournal,
    interact: (tester, app) async {
      await _openMore(tester, app);
      await _tapText(tester, 'Remove lock…');
    },
  ),
  _Screen(
    'lock_note',
    _closed,
    at: _plain,
    interact: (tester, app) async {
      await _openMore(tester, app);
      await _tapText(tester, 'Lock note…');
    },
  ),
  _Screen(
    'set_lock_password',
    _none,
    at: _plain,
    interact: (tester, app) async {
      await _openMore(tester, app);
      await _tapText(tester, 'Lock note…');
    },
  ),
  _Screen(
    'notebook_actions',
    _closed,
    interact: (tester, app) async {
      if (app.layoutClass.value != LayoutClass.triple) {
        await tester.tap(find.bySemanticsLabel('Library'));
        await tester.pumpAndSettle();
      }
      await tester.longPress(find.text('Journal').hitTestable().first);
      await tester.pumpAndSettle();
    },
  ),
  const _Screen('settings_locked_notes', _open, interact: _settings),
  _Screen(
    'change_password_wrong',
    _open,
    interact: (tester, app) async {
      await _settings(tester, app);
      await _tapText(tester, 'Change lock password…');
      await _type(tester, 'not the password', of: 3);
      await _type(tester, 'a new lock password', index: 1, of: 3);
      await _type(tester, 'a new lock password', index: 2, of: 3);
      await _tapText(tester, 'Change');
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
              name: 'lock_${viewport.slug}_${screen.name}_${brightness.name}',
              viewport: viewport,
              brightness: brightness,
              at: screen.at,
              library: screen.library,
              session: screen.session,
              interact: screen.interact,
            );
          });
        }
      });
    }
  }
}
