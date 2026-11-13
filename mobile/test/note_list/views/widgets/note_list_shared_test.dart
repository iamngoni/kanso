// Kanso — note_list_shared_test.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/note_list/views/widgets/note_list_pane.dart';

import '../../../helpers/browse_blocs.dart';
import '../../../helpers/fixture_shared_notebooks_repository.dart';
import '../../../helpers/library_fixtures.dart';
import '../../../helpers/pump_routed.dart';

const SharedEvent _synced = SharedEvent(
  noteIds: <String>[],
  invitations: 0,
  commentNotes: <String>[],
  liveNotes: <String>[],
  notebooks: <String>[MockIds.kanso],
  bookInvitations: 0,
  bookNotices: <String>[],
);

Future<void> _settle(WidgetTester tester) async {
  for (var i = 0; i < 6; i++) {
    await tester.pump(const Duration(milliseconds: 50));
  }
}

/// Pumps the list of the Kanso notebook, shared as [books] says.
Future<BrowseBlocs> _pumpKanso(
  WidgetTester tester,
  FixtureSharedNotebooksRepository books, {
  String place = MockIds.kanso,
}) async {
  final blocs = await BrowseBlocs.start();
  registerSharedNotebooks(books: books, shared: blocs.shared);
  final list = await blocs.listOf(ListPlace.notebook(id: place));
  await atMockNow(
    () => pumpRouted(tester, blocs.provide(const NoteListPane(), list: list)),
  );
  await atMockNow(() => _settle(tester));
  return blocs;
}

void main() {
  group('NoteListPane for a shared notebook', () {
    testWidgets('says who last edited each note, by their member name', (
      tester,
    ) async {
      final books = FixtureSharedNotebooksRepository()
        ..statuses[MockIds.kanso] = const BookSyncStatus(
          offline: false,
          waiting: 0,
          live: false,
        )
        ..editsOf[MockIds.kanso] = <NoteEditRow>[
          NoteEditRow(
            noteId: MockIds.syncDesign,
            name: 'dan',
            me: false,
            atMs: mockMs(10, 26, 10, 28),
          ),
        ];
      await _pumpKanso(tester, books);
      expect(find.textContaining('Edited by dan · '), findsOneWidget);
    });

    testWidgets('says when this account made the last edit', (tester) async {
      final books = FixtureSharedNotebooksRepository()
        ..statuses[MockIds.kanso] = const BookSyncStatus(
          offline: false,
          waiting: 0,
          live: false,
        )
        ..editsOf[MockIds.kanso] = <NoteEditRow>[
          NoteEditRow(
            noteId: MockIds.syncDesign,
            name: 'ngoni',
            me: true,
            atMs: mockMs(10, 26, 10, 28),
          ),
        ];
      await _pumpKanso(tester, books);
      expect(find.textContaining('Edited by you · '), findsOneWidget);
    });

    testWidgets('the subtitle says whether it is up to date', (tester) async {
      final books = FixtureSharedNotebooksRepository()
        ..statuses[MockIds.kanso] = const BookSyncStatus(
          offline: true,
          waiting: 3,
          live: false,
        );
      await _pumpKanso(tester, books);
      expect(
        find.textContaining('Offline · 3 changes waiting'),
        findsOneWidget,
      );
    });

    testWidgets('a sync that changes the notebook reads its status again', (
      tester,
    ) async {
      final books = FixtureSharedNotebooksRepository()
        ..statuses[MockIds.kanso] = const BookSyncStatus(
          offline: true,
          waiting: 1,
          live: false,
        );
      final blocs = await _pumpKanso(tester, books);
      expect(find.textContaining('1 change waiting'), findsOneWidget);
      books.statuses[MockIds.kanso] = const BookSyncStatus(
        offline: false,
        waiting: 0,
        live: false,
      );
      blocs.shared.eventsController.add(_synced);
      await atMockNow(() => _settle(tester));
      expect(find.textContaining('Up to date'), findsOneWidget);
      expect(find.textContaining('change waiting'), findsNothing);
    });

    testWidgets('a notebook not shared says neither', (tester) async {
      await _pumpKanso(tester, FixtureSharedNotebooksRepository());
      expect(find.textContaining('Edited by'), findsNothing);
      expect(find.textContaining('Up to date'), findsNothing);
    });
  });
}
