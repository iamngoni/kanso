// Kanso — viewport_goldens_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/layout_class.dart';
import 'package:kanso/settings/state/backup_notice/backup_notice_bloc.dart';
import 'package:kanso/shell/state/library_pane/library_pane_bloc.dart';

import '../../../helpers/app_golden.dart';
import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/fixture_library_repository.dart';
import '../../../helpers/kanso_viewport.dart';
import '../../../helpers/library_fixtures.dart';
import '../../../helpers/pump_browse_app.dart';
import '../../../helpers/real_themes.dart';
import '../../../helpers/seeded_appearance_bloc.dart';

/// One screen of the app, as a golden of the whole window.
class Screen {
  const Screen(
    this.name, {
    this.at = '/notes',
    this.library,
    this.interact,
    this.backupNotice,
    this.only,
  });

  /// The name in the golden's file.
  final String name;

  /// Where the app opens.
  final String at;

  /// Makes the library the screen needs, if not the plain mock one.
  final FixtureLibraryRepository Function()? library;

  /// What is done before the picture: a tap, a sheet.
  final Future<void> Function(WidgetTester tester, BrowseApp app)? interact;

  final Future<BackupNoticeBloc> Function()? backupNotice;

  /// The layout classes the screen exists in; all if null.
  final Set<LayoutClass>? only;
}

const ApplicationError _diskError = ApplicationError.storage(
  message: 'Disk error.',
);

Future<void> _hidePane(WidgetTester tester, BrowseApp app) async {
  app.blocs.libraryPane.add(const LibraryPaneEvent.toggled());
  await tester.pumpAndSettle();
}

Future<void> _openLibrary(WidgetTester tester, BrowseApp app) async {
  await tester.tap(find.bySemanticsLabel('Library'));
  await tester.pumpAndSettle();
}

Future<void> _openMore(WidgetTester tester, BrowseApp app) async {
  await tester.tap(find.bySemanticsLabel('More'));
  await tester.pumpAndSettle();
}

Future<void> _openFilter(WidgetTester tester, BrowseApp app) async {
  await tester.tap(find.bySemanticsLabel('Filter tasks'));
  await tester.pumpAndSettle();
}

Future<void> _select(WidgetTester tester, BrowseApp app) async {
  await tester.longPress(find.text('Sync design notes'));
  await tester.pumpAndSettle();
  await tester.tap(find.text('Sourdough starter'));
  await tester.pumpAndSettle();
}

final List<Screen> _screens = <Screen>[
  // The library is a pane beside the list on three panes, which is the
  // list's picture; on one or two it is a drawer.
  Screen(
    'library_drawer',
    interact: (tester, app) async {
      if (app.blocs.libraryPane.state.visible &&
          app.layoutClass.value == LayoutClass.triple) {
        await _hidePane(tester, app);
      }
      await _openLibrary(tester, app);
    },
  ),
  Screen(
    'library_failed',
    library: () => FixtureLibraryRepository()..notebooksFailure = _diskError,
  ),
  Screen(
    'library_loading',
    library: () =>
        FixtureLibraryRepository()..notebooksGate = Completer<void>(),
  ),
  const Screen('note_list'),
  Screen(
    'note_list_loading',
    library: () => FixtureLibraryRepository()..listGate = Completer<void>(),
  ),
  Screen(
    'note_list_empty',
    at: '/notebooks/${MockIds.someday}',
    library: () =>
        FixtureLibraryRepository()
          ..listResult = const Right<ApplicationError, NoteList>(
            NoteList(rows: <NoteRow>[], shape: mockShape),
          ),
  ),
  Screen(
    'note_list_failed',
    library: () =>
        FixtureLibraryRepository()
          ..listResult = const Left<ApplicationError, NoteList>(_diskError),
  ),
  const Screen('note_list_selecting', interact: _select),
  const Screen('note_list_options', interact: _openMore),
  const Screen('note_list_move', interact: _openMove),
  const Screen('note_list_swipe', interact: _swipe),
  const Screen('tasks', at: '/tasks'),
  Screen(
    'tasks_loading',
    at: '/tasks',
    library: () => FixtureLibraryRepository()..tasksGate = Completer<void>(),
  ),
  Screen(
    'tasks_empty',
    at: '/tasks',
    library: () =>
        FixtureLibraryRepository()
          ..tasksResult = const Right<ApplicationError, TaskBoard>(
            TaskBoard(groups: <TaskSection>[]),
          ),
  ),
  Screen(
    'tasks_failed',
    at: '/tasks',
    library: () =>
        FixtureLibraryRepository()
          ..tasksResult = const Left<ApplicationError, TaskBoard>(_diskError),
  ),
  const Screen('tasks_filter', at: '/tasks', interact: _openFilter),
  Screen('note', at: _noteAt(MockIds.syncDesign)),
  Screen('note_locked', at: _noteAt(MockIds.privateJournal)),
  Screen('note_not_found', at: _noteAt('note:gone')),
  Screen(
    'note_failed',
    at: _noteAt(MockIds.syncDesign),
    library: () => FixtureLibraryRepository()..noteFailure = _diskError,
  ),
  Screen(
    'note_loading',
    at: _noteAt(MockIds.syncDesign),
    library: () => FixtureLibraryRepository()..noteGate = Completer<void>(),
  ),
  const Screen('search_idle', at: '/search'),
  const Screen('search', at: '/search?q=sync'),
  Screen(
    'search_empty',
    at: '/search?q=zzz',
    library: () =>
        FixtureLibraryRepository()
          ..searchResult = const Right<ApplicationError, List<Hit>>(<Hit>[]),
  ),
  Screen(
    'search_failed',
    at: '/search?q=sync',
    library: () =>
        FixtureLibraryRepository()
          ..searchResult = const Left<ApplicationError, List<Hit>>(_diskError),
  ),
  Screen(
    'search_loading',
    at: '/search?q=sync',
    library: () => FixtureLibraryRepository()..searchGate = Completer<void>(),
  ),
  Screen(
    'search_locked',
    at: '/search?q=journal',
    library: () =>
        FixtureLibraryRepository()
          ..searchResult = Right<ApplicationError, List<Hit>>(<Hit>[
            mockHit(mockRows.firstWhere((row) => row.locked), 'journal'),
            mockHits.first,
          ]),
  ),
  Screen(
    'settings',
    interact: (tester, app) => _push(tester, app, '/settings'),
  ),
  Screen(
    'settings_fonts',
    interact: (tester, app) async {
      await _push(tester, app, '/settings');
      await _push(tester, app, '/settings/fonts');
    },
  ),
  Screen(
    'settings_theme',
    interact: (tester, app) async {
      await _push(tester, app, '/settings');
      await _push(tester, app, '/settings/theme');
    },
  ),
  Screen(
    'settings_licences',
    interact: (tester, app) async {
      await _push(tester, app, '/settings');
      await _push(tester, app, '/settings/licences');
    },
  ),
  Screen(
    'backup_notice',
    backupNotice: () async {
      final notice = BackupNoticeBloc(ForgetfulPreferencesRepository())
        ..add(const BackupNoticeEvent.started());
      await notice.stream.firstWhere((state) => state.visible);
      closeBlocLater(notice);
      return notice;
    },
  ),
];

String _noteAt(String id) => '/notes/note/${Uri.encodeComponent(id)}';

Future<void> _push(WidgetTester tester, BrowseApp app, String path) async {
  unawaited(app.router.push<void>(path));
  await tester.pumpAndSettle();
}

Future<void> _openMove(WidgetTester tester, BrowseApp app) async {
  await tester.longPress(find.text('Lisbon in spring'));
  await tester.pumpAndSettle();
  await tester.tap(find.bySemanticsLabel('Move'));
  await tester.pumpAndSettle();
}

Future<void> _swipe(WidgetTester tester, BrowseApp app) async {
  await tester.drag(find.text('Sourdough starter'), const Offset(120, 0));
}

/// The screens a phone has no picture of yet: the others are in the tests of
/// their panes, at the phone's size.
const Set<String> _phoneScreens = <String>{
  'settings_licences',
  'tasks',
  'tasks_loading',
  'tasks_empty',
  'tasks_failed',
  'tasks_filter',
};

void main() {
  loadRealThemes();

  for (final brightness in Brightness.values) {
    for (final screen in _screens.where(
      (s) => _phoneScreens.contains(s.name),
    )) {
      testWidgets('phone, ${brightness.name}, ${screen.name}', (tester) async {
        await expectAppGoldenOnBothPlatforms(
          tester,
          name: 'phone_${screen.name}_${brightness.name}',
          viewport: KansoViewport.phone,
          brightness: brightness,
          at: screen.at,
          library: screen.library,
          interact: screen.interact,
        );
      });
    }
  }

  for (final viewport in KansoViewport.goldenSet) {
    for (final brightness in Brightness.values) {
      group('${viewport.slug}, ${brightness.name}', () {
        for (final screen in _screens) {
          if (screen.only != null && !screen.only!.contains(viewport.layout)) {
            continue;
          }
          testWidgets(screen.name, (tester) async {
            await expectAppGoldenOnBothPlatforms(
              tester,
              name: '${viewport.slug}_${screen.name}_${brightness.name}',
              viewport: viewport,
              brightness: brightness,
              at: screen.at,
              library: screen.library,
              backupNotice: screen.backupNotice,
              interact: screen.interact,
            );
          });
        }
      });
    }
  }
}
