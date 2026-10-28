// Kanso — kanso_shortcuts_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart' show EditableText;
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/views/widgets/kanso_drawer_scope.dart';
import 'package:kanso/note_list/views/pages/note_list_page.dart';
import 'package:kanso/shell/views/widgets/library_navigation.dart';

import '../../../helpers/fixture_library_repository.dart';
import '../../../helpers/library_fixtures.dart';
import '../../../helpers/pump_browse_app.dart';
import '../../../helpers/pump_routed.dart';

const Size _landscape = Size(1180, 820);
const Size _portrait = Size(820, 1180);

/// Presses [key] with the shortcut modifier of the platform under test, and
/// shift or alt if asked, and lets go.
Future<void> _shortcut(
  WidgetTester tester,
  LogicalKeyboardKey key, {
  bool shift = false,
  bool alt = false,
}) async {
  final secondary = defaultTargetPlatform == TargetPlatform.iOS
      ? LogicalKeyboardKey.metaLeft
      : LogicalKeyboardKey.controlLeft;
  await tester.sendKeyDownEvent(secondary);
  if (shift) {
    await tester.sendKeyDownEvent(LogicalKeyboardKey.shiftLeft);
  }
  if (alt) {
    await tester.sendKeyDownEvent(LogicalKeyboardKey.altLeft);
  }
  await tester.sendKeyEvent(key);
  if (alt) {
    await tester.sendKeyUpEvent(LogicalKeyboardKey.altLeft);
  }
  if (shift) {
    await tester.sendKeyUpEvent(LogicalKeyboardKey.shiftLeft);
  }
  await tester.sendKeyUpEvent(secondary);
  // The blocs behind the shortcut take a few turns of the event loop.
  for (var i = 0; i < 5; i++) {
    await tester.pump();
  }
  await tester.pumpAndSettle();
}

String _noteIn(String address) => Uri.decodeComponent(address.split('/').last);

void main() {
  group('KansoShortcuts', () {
    testWidgets('secondary + comma opens settings', (tester) async {
      final app = await pumpBrowseApp(tester, _landscape);
      await _shortcut(tester, LogicalKeyboardKey.comma);
      expect(topOf(app.router), '/settings');
    });

    testWidgets('secondary + shift + F searches all notes', (tester) async {
      final app = await pumpBrowseApp(tester, _landscape, at: '/pinned');
      await _shortcut(tester, LogicalKeyboardKey.keyF, shift: true);
      expect(app.where, '/search');
    });

    testWidgets('secondary + F alone does not, as on the desktop', (
      tester,
    ) async {
      final app = await pumpBrowseApp(tester, _landscape, at: '/pinned');
      await _shortcut(tester, LogicalKeyboardKey.keyF);
      expect(app.where, '/pinned');
    });

    testWidgets('the platform’s own key is the secondary one', (
      tester,
    ) async {
      debugDefaultTargetPlatformOverride = TargetPlatform.iOS;
      try {
        final app = await pumpBrowseApp(tester, _landscape);
        await _shortcut(tester, LogicalKeyboardKey.comma);
        expect(topOf(app.router), '/settings');
      } finally {
        debugDefaultTargetPlatformOverride = null;
      }
    });

    testWidgets('secondary + option + down and up step through the list', (
      tester,
    ) async {
      final app = await pumpBrowseApp(tester, _landscape);
      final first = mockRows.first.id;
      final second = mockRows[1].id;

      await _shortcut(tester, LogicalKeyboardKey.arrowDown, alt: true);
      expect(_noteIn(app.where), first);
      await _shortcut(tester, LogicalKeyboardKey.arrowDown, alt: true);
      expect(_noteIn(app.where), second);
      await _shortcut(tester, LogicalKeyboardKey.arrowUp, alt: true);
      expect(_noteIn(app.where), first);
      // At the top, up stays.
      await _shortcut(tester, LogicalKeyboardKey.arrowUp, alt: true);
      expect(_noteIn(app.where), first);
    });

    testWidgets('secondary + [ and ] go back and forward through the notes '
        'opened', (tester) async {
      final library = FixtureLibraryRepository()
        ..notes.addAll(readableMockNotes());
      final app = await pumpBrowseApp(tester, _landscape, library: library);
      final first = mockRows.first.id;
      final second = mockRows[1].id;
      await _shortcut(tester, LogicalKeyboardKey.arrowDown, alt: true);
      await _shortcut(tester, LogicalKeyboardKey.arrowDown, alt: true);
      expect(_noteIn(app.where), second);

      await _shortcut(tester, LogicalKeyboardKey.bracketLeft);
      expect(_noteIn(app.where), first);
      await _shortcut(tester, LogicalKeyboardKey.bracketRight);
      expect(_noteIn(app.where), second);
    });

    testWidgets('secondary + shift + P pins the open note, and unpins it', (
      tester,
    ) async {
      final app = await pumpBrowseApp(tester, _landscape);
      await _shortcut(tester, LogicalKeyboardKey.arrowDown, alt: true);
      final open = _noteIn(app.where);
      await _shortcut(tester, LogicalKeyboardKey.keyP, shift: true);
      expect(app.blocs.library.changesMade, hasLength(1));
      expect(app.blocs.library.changesMade.single.$1, <String>[open]);
      expect(
        app.blocs.library.changesMade.single.$2,
        const NoteChange.togglePin(),
      );
    });

    testWidgets('secondary + shift + P with no note open does nothing', (
      tester,
    ) async {
      final app = await pumpBrowseApp(tester, _landscape);
      await _shortcut(tester, LogicalKeyboardKey.keyP, shift: true);
      expect(app.blocs.library.changesMade, isEmpty);
    });

    testWidgets('secondary + backslash hides the library pane of three, and '
        'shows it again', (tester) async {
      await pumpBrowseApp(tester, _landscape);
      expect(find.byType(LibraryNavigation), findsOneWidget);
      expect(tester.getTopLeft(find.byType(LibraryNavigation)).dx, 0);
      expect(
        KansoDrawerScope.maybeOf(tester.element(find.byType(NoteListPage))),
        isNull,
      );

      await _shortcut(tester, LogicalKeyboardKey.backslash);
      // Hidden, the library is the drawer of the two panes left, closed.
      expect(find.byType(LibraryNavigation), findsNothing);
      expect(
        KansoDrawerScope.maybeOf(tester.element(find.byType(NoteListPage))),
        isNotNull,
      );

      await _shortcut(tester, LogicalKeyboardKey.backslash);
      expect(tester.getTopLeft(find.byType(LibraryNavigation)).dx, 0);
      expect(
        KansoDrawerScope.maybeOf(tester.element(find.byType(NoteListPage))),
        isNull,
      );
    });

    testWidgets('secondary + backslash opens and closes the drawer of two '
        'panes', (tester) async {
      await pumpBrowseApp(tester, _portrait);
      expect(find.byType(LibraryNavigation), findsNothing);
      await _shortcut(tester, LogicalKeyboardKey.backslash);
      expect(find.byType(LibraryNavigation), findsOneWidget);
      await _shortcut(tester, LogicalKeyboardKey.backslash);
      expect(find.byType(LibraryNavigation), findsNothing);
    });

    testWidgets('secondary + N makes a note in the place being listed and '
        'opens it', (tester) async {
      final app = await pumpBrowseApp(
        tester,
        _landscape,
        at: '/notebooks/${MockIds.kanso}',
      );
      await _shortcut(tester, LogicalKeyboardKey.keyN);
      final made = app.blocs.library.created.single;
      expect(made.place, const ListPlace.notebook(id: MockIds.kanso));
      expect(made.pinned, isFalse);
      expect(
        Uri.decodeComponent(app.where),
        '/notebooks/${MockIds.kanso}/note/note:new-1',
      );
    });

    testWidgets('secondary + shift + N asks for a notebook to make', (
      tester,
    ) async {
      final app = await pumpBrowseApp(tester, _landscape, at: '/pinned');
      await _shortcut(tester, LogicalKeyboardKey.keyN, shift: true);
      expect(find.text('New notebook'), findsWidgets);
      await tester.enterText(find.byType(EditableText).last, 'Recipes');
      await tester.pump();
      await tester.tap(find.text('Create'));
      await tester.pumpAndSettle();
      expect(app.blocs.library.notebooksCreated.single.$1, 'Recipes');
      expect(app.where, startsWith('/notebooks/'));
      expect(app.blocs.library.created, isEmpty);
    });

    testWidgets('secondary + N in Pinned makes a pinned note, and from a '
        'search a note in All Notes', (tester) async {
      final app = await pumpBrowseApp(tester, _landscape, at: '/pinned');
      await _shortcut(tester, LogicalKeyboardKey.keyN);
      expect(app.blocs.library.created.single.pinned, isTrue);
      expect(app.where, startsWith('/pinned/note/'));
      app.router.go('/search?q=tea');
      await tester.pumpAndSettle();
      await _shortcut(tester, LogicalKeyboardKey.keyN);
      expect(app.blocs.library.created.last.place, const ListPlace.all());
      expect(app.where, startsWith('/notes/note/'));
    });

    testWidgets('has no shortcut for what is not built: find in a note, the '
        'modes of a note on the list, the command palette', (tester) async {
      final app = await pumpBrowseApp(tester, _landscape, at: '/pinned');
      await _shortcut(tester, LogicalKeyboardKey.digit1);
      await _shortcut(tester, LogicalKeyboardKey.digit2);
      await _shortcut(tester, LogicalKeyboardKey.digit3);
      await _shortcut(tester, LogicalKeyboardKey.keyK);
      expect(app.where, '/pinned');
      expect(app.blocs.library.changesMade, isEmpty);
    });
  });
}
