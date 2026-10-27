// Kanso — notebook_lock_menu_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/browse_location.dart';
import 'package:kanso/core/views/widgets/kanso_lock_mark.dart';
import 'package:kanso/library/views/widgets/library_pane.dart';
import 'package:kanso/note_list/views/widgets/note_list_pane.dart';
import 'package:kanso/note_list/views/widgets/notebook_gate_host.dart';

import '../../../helpers/browse_blocs.dart';
import '../../../helpers/fixture_library_repository.dart';
import '../../../helpers/fixture_session_repository.dart';
import '../../../helpers/library_fixtures.dart';
import '../../../helpers/pump_routed.dart';

List<NotebookRow> _books({
  bool journalLocked = false,
  bool kansoLocked = false,
  bool projectsLocked = false,
}) => <NotebookRow>[
  notebookRow(MockIds.inbox, name: 'Inbox', count: 12),
  notebookRow(
    MockIds.journal,
    name: 'Journal',
    count: 48,
    locked: journalLocked,
  ),
  notebookRow(
    MockIds.projects,
    name: 'Projects',
    count: 23,
    locked: projectsLocked,
  ),
  notebookRow(
    MockIds.kanso,
    name: 'Kanso',
    count: 17,
    depth: 1,
    parentId: MockIds.projects,
    locked: kansoLocked,
  ),
];

Future<BrowseBlocs> _pumpLibrary(
  WidgetTester tester, {
  required List<NotebookRow> books,
  FixtureSessionRepository? session,
}) async {
  final library = FixtureLibraryRepository()..notebookRows = books;
  final blocs = await BrowseBlocs.start(
    library,
    null,
    null,
    session ?? FixtureSessionRepository(),
  );
  await pumpRouted(
    tester,
    blocs.provide(
      const LibraryPane(location: BrowseLocation.place(ListPlace.all())),
    ),
  );
  await tester.pump();
  return blocs;
}

void main() {
  testWidgets('a locked notebook is marked with a lock in the library', (
    tester,
  ) async {
    await _pumpLibrary(tester, books: _books(journalLocked: true));
    expect(find.byType(KansoLockMark), findsOneWidget);
    expect(
      find.descendant(
        of: find.widgetWithText(Row, 'Journal'),
        matching: find.byType(KansoLockMark),
      ),
      findsOneWidget,
    );
  });

  testWidgets('a long press on an unlocked notebook offers Lock Notebook…', (
    tester,
  ) async {
    await _pumpLibrary(tester, books: _books());
    await tester.longPress(find.text('Journal'));
    await tester.pumpAndSettle();
    expect(find.text('Lock Notebook…'), findsOneWidget);
    expect(find.text('Remove Notebook Lock…'), findsNothing);
  });

  testWidgets('with the notes open, locking only confirms', (tester) async {
    final session = FixtureSessionRepository(unlocked: true);
    await _pumpLibrary(tester, books: _books(), session: session);
    await tester.longPress(find.text('Journal'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Lock Notebook…'));
    await tester.pumpAndSettle();
    expect(find.text('Lock this notebook?'), findsOneWidget);
    await tester.tap(find.text('Lock'));
    await tester.pumpAndSettle();
    expect(session.calls, contains('lockNotebook:${MockIds.journal}'));
    expect(find.text('Lock this notebook?'), findsNothing);
  });

  testWidgets('with the notes closed it asks for the password', (tester) async {
    final session = FixtureSessionRepository();
    await _pumpLibrary(tester, books: _books(), session: session);
    await tester.longPress(find.text('Journal'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Lock Notebook…'));
    await tester.pumpAndSettle();
    expect(find.text('Lock notebook'), findsOneWidget);
    await tester.enterText(find.byType(EditableText), 'nope nope');
    await tester.tap(find.text('Lock'));
    await tester.pumpAndSettle();
    expect(find.text("That isn't the lock password."), findsOneWidget);
    await tester.enterText(find.byType(EditableText), session.password);
    await tester.tap(find.text('Lock'));
    await tester.pumpAndSettle();
    expect(find.text('Lock notebook'), findsNothing);
  });

  testWidgets('the first lock of anything makes the lock password', (
    tester,
  ) async {
    final session = FixtureSessionRepository(hasLockPassword: false);
    await _pumpLibrary(tester, books: _books(), session: session);
    await tester.longPress(find.text('Journal'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Lock Notebook…'));
    await tester.pumpAndSettle();
    expect(find.text('Set a lock password'), findsOneWidget);
  });

  testWidgets('a notebook with its own lock can have it removed', (
    tester,
  ) async {
    final session = FixtureSessionRepository(unlocked: true);
    await _pumpLibrary(
      tester,
      books: _books(journalLocked: true),
      session: session,
    );
    await tester.longPress(find.text('Journal'));
    await tester.pumpAndSettle();
    expect(find.text('Lock Notebook…'), findsNothing);
    await tester.tap(find.text('Remove Notebook Lock…'));
    await tester.pumpAndSettle();
    expect(find.text("Remove the notebook's lock?"), findsOneWidget);
    await tester.tap(find.text('Remove Lock'));
    await tester.pumpAndSettle();
    expect(session.calls, contains('removeNotebookLock:${MockIds.journal}'));
  });

  testWidgets('removing a lock with the notes closed unlocks first', (
    tester,
  ) async {
    final session = FixtureSessionRepository();
    await _pumpLibrary(
      tester,
      books: _books(journalLocked: true),
      session: session,
    );
    await tester.longPress(find.text('Journal'));
    await tester.pumpAndSettle();
    expect(find.text('Unlock notebook…'), findsOneWidget);
    await tester.tap(find.text('Remove Notebook Lock…'));
    await tester.pumpAndSettle();
    expect(find.text('Unlock notebook'), findsOneWidget);
    expect(
      find.text('“Journal” is locked. Its notes open with your lock password.'),
      findsOneWidget,
    );
    await tester.enterText(find.byType(EditableText), session.password);
    await tester.tap(find.text('Unlock'));
    await tester.pumpAndSettle();
    // The offer of Face ID comes first, once; leave it.
    if (find.text('Use Face ID?').evaluate().isNotEmpty) {
      await tester.tap(find.text('Not now'));
      await tester.pumpAndSettle();
    }
    expect(find.text("Remove the notebook's lock?"), findsOneWidget);
  });

  testWidgets('a notebook locked through its parent has no lock item', (
    tester,
  ) async {
    await _pumpLibrary(tester, books: _books(projectsLocked: true));
    await tester.longPress(find.text('Kanso').last);
    await tester.pumpAndSettle();
    expect(find.text('Lock Notebook…'), findsNothing);
    expect(find.text('Remove Notebook Lock…'), findsNothing);
  });

  group('showing the notes of a locked notebook', () {
    Future<BrowseBlocs> list(
      WidgetTester tester, {
      FixtureSessionRepository? session,
    }) async {
      final library = FixtureLibraryRepository()
        ..notebookRows = _books(journalLocked: true);
      final blocs = await BrowseBlocs.start(
        library,
        null,
        null,
        session ?? FixtureSessionRepository(),
      );
      const place = ListPlace.notebook(id: MockIds.journal);
      final bloc = await blocs.listOf(place);
      await pumpRouted(
        tester,
        blocs.provide(
          const NotebookGateHost(place: place, child: NoteListPane()),
          list: bloc,
        ),
      );
      await tester.pumpAndSettle();
      return blocs;
    }

    testWidgets('asks once for the password when biometrics are off', (
      tester,
    ) async {
      await list(tester);
      expect(find.text('Unlock notebook'), findsOneWidget);
    });
  });
}
