// Kanso — shared_notebooks_section_test.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/sharing/views/widgets/share_notebook_sheet.dart';
import 'package:kanso/sharing/views/widgets/shared_notebooks_section.dart';

import '../../../helpers/fixture_shared_notebooks_repository.dart';
import '../../../helpers/fixture_shared_notes_repository.dart';
import '../../../helpers/pump_routed.dart';

Future<void> _settle(WidgetTester tester) async {
  for (var i = 0; i < 10; i++) {
    await tester.pump(const Duration(milliseconds: 50));
  }
}

SharedBookRow _book(
  String notebookId, {
  String name = 'Studio',
  List<String> members = const <String>['ngoni', 'dan', 'eve'],
  BookSyncStatus status = const BookSyncStatus(
    offline: false,
    waiting: 0,
    live: false,
  ),
}) => SharedBookRow(
  shareId: 's_$notebookId',
  notebookId: notebookId,
  name: name,
  myRole: BookRole.editor,
  owner: 'ngoni',
  members: members,
  status: status,
);

BookInvitationRow _invitation(String shareId, {bool invited = true}) =>
    BookInvitationRow(
      shareId: shareId,
      from: 'ana',
      role: BookRole.editor,
      invited: invited,
      atMs: 0,
    );

const String _gone =
    'You can no longer open "Studio". Your copy is in "Studio (no longer '
    'shared)".';

const SharedEvent _news = SharedEvent(
  noteIds: <String>[],
  invitations: 0,
  commentNotes: <String>[],
  liveNotes: <String>[],
  notebooks: <String>['book:1'],
  bookInvitations: 0,
  bookNotices: <String>[_gone],
);

typedef _Pumped = (
  FixtureSharedNotebooksRepository,
  FixtureSharedNotesRepository,
  GoRouter,
);

Future<_Pumped> _pump(
  WidgetTester tester,
  FixtureSharedNotebooksRepository books,
) async {
  final shared = FixtureSharedNotesRepository();
  registerSharedNotebooks(books: books, shared: shared);
  final router = await pumpRouted(
    tester,
    const SingleChildScrollView(
      child: SharedNotebooksSection(selectedNotebook: 'book:1'),
    ),
  );
  await _settle(tester);
  return (books, shared, router);
}

void main() {
  group('SharedNotebooksSection', () {
    testWidgets('shows nothing while nothing is shared', (tester) async {
      await _pump(tester, FixtureSharedNotebooksRepository());
      expect(find.text('Shared'), findsNothing);
    });

    testWidgets('shows nothing signed out', (tester) async {
      final books = FixtureSharedNotebooksRepository()
        ..books = <SharedBookRow>[_book('book:1')]
        ..signedOut = true;
      await _pump(tester, books);
      expect(find.text('Shared'), findsNothing);
      expect(find.text('Studio'), findsNothing);
    });

    testWidgets('lists the shared notebooks with whether each is up to date', (
      tester,
    ) async {
      final books = FixtureSharedNotebooksRepository()
        ..books = <SharedBookRow>[
          _book('book:1'),
          _book(
            'book:2',
            name: 'Garden',
            status: const BookSyncStatus(
              offline: true,
              waiting: 4,
              live: false,
            ),
          ),
          _book(
            'book:3',
            name: 'Trip',
            status: const BookSyncStatus(
              offline: false,
              waiting: 1,
              live: true,
            ),
          ),
        ];
      await _pump(tester, books);
      expect(find.text('Shared'), findsOneWidget);
      expect(find.text('Studio'), findsOneWidget);
      expect(find.text('Up to date'), findsOneWidget);
      expect(find.text('Garden'), findsOneWidget);
      expect(find.text('Offline · 4 changes waiting'), findsOneWidget);
      expect(find.text('1 change waiting'), findsOneWidget);
      // Faces: the first letter of each person's name.
      expect(find.text('D'), findsWidgets);
      expect(
        find.bySemanticsLabel('Studio, 3 people, Up to date'),
        findsOneWidget,
      );
    });

    testWidgets('a tap opens the notebook', (tester) async {
      final books = FixtureSharedNotebooksRepository()
        ..books = <SharedBookRow>[_book('book:7', name: 'Trip')];
      final (_, _, router) = await _pump(tester, books);
      await tester.tap(find.text('Trip'));
      await _settle(tester);
      expect(
        router.routerDelegate.currentConfiguration.uri.path,
        '/notebooks/${Uri.encodeComponent('book:7')}',
      );
    });

    testWidgets('a long press opens its Share sheet', (tester) async {
      final books = FixtureSharedNotebooksRepository()
        ..books = <SharedBookRow>[_book('book:1')]
        ..shares['book:1'] = bookInfo('book:1');
      await _pump(tester, books);
      await tester.longPress(find.text('Studio'));
      await _settle(tester);
      expect(find.byType(ShareNotebookView), findsOneWidget);
      expect(find.text('Share “Studio”'), findsOneWidget);
    });

    testWidgets('accepts an invitation and opens the notebook', (
      tester,
    ) async {
      final books = FixtureSharedNotebooksRepository()
        ..invitationList = <BookInvitationRow>[_invitation('s9')];
      final (_, _, router) = await _pump(tester, books);
      expect(find.text('Shared'), findsOneWidget);
      expect(
        find.text('ana invited you to a notebook · Can edit'),
        findsOneWidget,
      );
      await tester.tap(find.text('Accept'));
      await _settle(tester);
      expect(books.calls, contains('accept s9'));
      expect(find.text('Accept'), findsNothing);
      expect(
        router.routerDelegate.currentConfiguration.uri.path,
        '/notebooks/${Uri.encodeComponent('book:s9')}',
      );
    });

    testWidgets('declines an invitation', (tester) async {
      final books = FixtureSharedNotebooksRepository()
        ..invitationList = <BookInvitationRow>[_invitation('s9')];
      await _pump(tester, books);
      await tester.tap(find.text('Decline'));
      await _settle(tester);
      expect(books.calls, contains('decline s9'));
      expect(find.textContaining('invited you'), findsNothing);
    });

    testWidgets('one joined on another device offers Open here, no Decline', (
      tester,
    ) async {
      final books = FixtureSharedNotebooksRepository()
        ..invitationList = <BookInvitationRow>[
          _invitation('s9', invited: false),
        ];
      await _pump(tester, books);
      expect(
        find.text('A notebook ana shared with you · Can edit'),
        findsOneWidget,
      );
      expect(find.text('Open here'), findsOneWidget);
      expect(find.text('Decline'), findsNothing);
    });

    testWidgets('says when a notebook stopped being shared, until read', (
      tester,
    ) async {
      final books = FixtureSharedNotebooksRepository()
        ..books = <SharedBookRow>[_book('book:2', name: 'Garden')];
      final (_, shared, _) = await _pump(tester, books);
      shared.eventsController.add(_news);
      await _settle(tester);
      expect(
        find.textContaining('Your copy is in "Studio (no longer shared)"'),
        findsOneWidget,
      );
      await tester.tap(find.text('OK'));
      await _settle(tester);
      expect(find.textContaining('no longer shared'), findsNothing);
    });

    testWidgets('a sync that changes a notebook reads the list again', (
      tester,
    ) async {
      final books = FixtureSharedNotebooksRepository()
        ..books = <SharedBookRow>[_book('book:1')];
      final (_, shared, _) = await _pump(tester, books);
      books.books = <SharedBookRow>[
        _book(
          'book:1',
          status: const BookSyncStatus(offline: true, waiting: 0, live: false),
        ),
      ];
      shared.eventsController.add(_news);
      await _settle(tester);
      expect(find.text('Offline'), findsOneWidget);
    });

    testWidgets('says to update where an invitation would be, once the '
        'server refused this Kanso', (tester) async {
      const line = 'Update Kanso to open this shared notebook.';
      final books = FixtureSharedNotebooksRepository()..updateLineText = line;
      await _pump(tester, books);
      expect(find.text('Shared'), findsOneWidget);
      expect(find.text(line), findsOneWidget);
      expect(
        find.byKey(const ValueKey<String>('book-update-needed')),
        findsOneWidget,
      );
    });

    testWidgets('a refusal that comes with a later sync shows the line', (
      tester,
    ) async {
      const line = 'Update Kanso to open this shared notebook.';
      final books = FixtureSharedNotebooksRepository()
        ..books = <SharedBookRow>[_book('book:1')];
      final (_, shared, _) = await _pump(tester, books);
      expect(find.text(line), findsNothing);
      books.updateLineText = line;
      shared.eventsController.add(_news);
      await _settle(tester);
      expect(find.text(line), findsOneWidget);
      expect(find.text('Studio'), findsOneWidget);
    });
  });
}
