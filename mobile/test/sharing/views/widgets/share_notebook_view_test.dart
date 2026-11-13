// Kanso — share_notebook_view_test.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/sharing/state/share_notebook/share_notebook_bloc.dart';
import 'package:kanso/core/views/widgets/kanso_button.dart';
import 'package:kanso/sharing/views/widgets/book_invite_bar.dart';
import 'package:kanso/sharing/views/widgets/book_member_row.dart';
import 'package:kanso/sharing/views/widgets/share_notebook_sheet.dart';

import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/fixture_shared_notebooks_repository.dart';
import '../../../helpers/fixture_shared_notes_repository.dart';
import '../../../helpers/pump_routed.dart';
import '../../../helpers/sharing_fixtures.dart';

class _Sheet {
  _Sheet(this.books, this.people, this.bloc);

  final FixtureSharedNotebooksRepository books;
  final FixtureSharedNotesRepository people;
  final ShareNotebookBloc bloc;
}

Future<void> _settle(WidgetTester tester) async {
  for (var i = 0; i < 10; i++) {
    await tester.pump(const Duration(milliseconds: 50));
  }
}

Future<_Sheet> _pump(
  WidgetTester tester, {
  FixtureSharedNotebooksRepository? books,
  FixtureSharedNotesRepository? people,
  String notebookId = 'book:1',
}) async {
  final notebooks = books ?? FixtureSharedNotebooksRepository();
  final shared = people ?? FixtureSharedNotesRepository();
  // The sheet reads its confirm words through the app's GetIt.
  registerSharedNotebooks(books: notebooks, shared: shared);
  final bloc = ShareNotebookBloc(notebooks, shared)
    ..add(ShareNotebookEvent.started(notebookId));
  closeBlocLater(bloc);
  await pumpRouted(
    tester,
    BlocProvider<ShareNotebookBloc>.value(
      value: bloc,
      child: const SizedBox(height: 820, child: ShareNotebookView()),
    ),
  );
  await _settle(tester);
  return _Sheet(notebooks, shared, bloc);
}

Future<void> _tapText(WidgetTester tester, String text) async {
  final target = find.text(text).last;
  await tester.ensureVisible(target);
  await tester.pump();
  await tester.tap(target);
  await _settle(tester);
}

/// A notebook shared by this account with dan, whose menu offers [roles].
FixtureSharedNotebooksRepository _ownedWithDan({
  BookRole myRole = BookRole.owner,
  List<BookRole> roles = const <BookRole>[
    BookRole.manager,
    BookRole.editor,
    BookRole.commenter,
    BookRole.viewer,
  ],
  bool removable = true,
}) {
  final books = FixtureSharedNotebooksRepository();
  books.shares['book:1'] = bookInfo(
    'book:1',
    myRole: myRole,
    owner: myRole == BookRole.owner ? 'ngoni' : 'ana',
    members: <BookMemberRow>[
      if (myRole != BookRole.owner) bookMember('ana', role: BookRole.owner),
      bookMember('ngoni', role: myRole, me: true),
      bookMember('dan', roles: roles, removable: removable, verified: true),
    ],
  );
  return books;
}

void main() {
  group('ShareNotebookView for a notebook that is not shared', () {
    testWidgets('says what sharing does, then shares with the first person', (
      tester,
    ) async {
      final books = FixtureSharedNotebooksRepository()
        ..checks['book:1'] = const BookShareCheck(
          name: 'Studio',
          notes: 12,
          locked: 2,
          lockedName: 'Studio (locked)',
          leftOut: <String>['Plan'],
        );
      final people = FixtureSharedNotesRepository()
        ..people['dan'] = PersonLookup.found(personRow('dan', verified: true));
      await _pump(tester, books: books, people: people);

      expect(find.textContaining('Invite people by their Kanso'), findsOne);
      expect(find.text('12 notes go in.'), findsOneWidget);
      expect(
        find.textContaining('2 locked notes will stay in a private'),
        findsOneWidget,
      );
      expect(find.textContaining('Not shared: Plan.'), findsOneWidget);
      expect(find.byType(BookInviteBar), findsOneWidget);
      expect(find.text('People'), findsNothing);

      await tester.enterText(find.byType(EditableText), 'dan');
      await _tapText(tester, 'Share');

      expect(books.calls, contains('share book:1 dan editor'));
      expect(
        find.text(
          'Shared “Studio” with dan. It shows for them once they accept.',
        ),
        findsOneWidget,
      );
      expect(find.text('People'), findsOneWidget);
      expect(find.text('dan'), findsOneWidget);
      expect(find.text("Invited, hasn't accepted yet"), findsOneWidget);
      // The next person is invited, not shared with.
      expect(find.text('Invite'), findsOneWidget);
      expect(
        tester.widget<EditableText>(find.byType(EditableText)).controller.text,
        isEmpty,
      );
    });

    testWidgets('shows how far sharing has got while notes go up', (
      tester,
    ) async {
      final gate = Completer<void>();
      final books = FixtureSharedNotebooksRepository()
        ..shareProgress = const <(int, int)>[(120, 340)]
        ..shareGate = gate;
      final people = FixtureSharedNotesRepository()
        ..people['dan'] = PersonLookup.found(personRow('dan', verified: true));
      await _pump(tester, books: books, people: people);
      await tester.enterText(find.byType(EditableText), 'dan');
      await _tapText(tester, 'Share');
      expect(find.text('Sharing Studio… 120 of 340'), findsOneWidget);
      gate.complete();
      await _settle(tester);
      // Done: the progress line is gone.
      expect(find.textContaining('120 of 340'), findsNothing);
      expect(find.text('People'), findsOneWidget);
    });

    testWidgets('the role chosen is the role given', (tester) async {
      final books = FixtureSharedNotebooksRepository();
      final people = FixtureSharedNotesRepository()
        ..people['dan'] = PersonLookup.found(personRow('dan', verified: true));
      await _pump(tester, books: books, people: people);
      await _tapText(tester, 'Can edit');
      expect(find.text('Role for the person you invite'), findsOneWidget);
      await _tapText(tester, 'Can view');
      await tester.enterText(find.byType(EditableText), 'dan');
      await _tapText(tester, 'Share');
      expect(books.calls, contains('share book:1 dan viewer'));
    });

    testWidgets('asks to compare a new person’s fingerprint first', (
      tester,
    ) async {
      final books = FixtureSharedNotebooksRepository();
      final people = FixtureSharedNotesRepository()
        ..people['bo'] = PersonLookup.found(personRow('bo'));
      await _pump(tester, books: books, people: people);
      await tester.enterText(find.byType(EditableText), 'bo');
      await _tapText(tester, 'Share');
      expect(
        find.textContaining('You’re sharing with bo for the first time'),
        findsOneWidget,
      );
      expect(books.calls.where((c) => c.startsWith('share')), isEmpty);

      await _tapText(tester, 'Mark as verified and invite');
      expect(people.calls, contains('verify a_bo'));
      expect(books.calls, contains('share book:1 bo editor'));
      expect(find.text('People'), findsOneWidget);
    });

    testWidgets('Back from the fingerprint question shares nothing', (
      tester,
    ) async {
      final books = FixtureSharedNotebooksRepository();
      final people = FixtureSharedNotesRepository()
        ..people['bo'] = PersonLookup.found(personRow('bo'));
      await _pump(tester, books: books, people: people);
      await tester.enterText(find.byType(EditableText), 'bo');
      await _tapText(tester, 'Share');
      await _tapText(tester, 'Back');
      expect(find.byType(BookInviteBar), findsOneWidget);
      expect(books.calls.where((c) => c.startsWith('share')), isEmpty);
    });

    testWidgets('says to type a username when none was typed', (tester) async {
      final books = FixtureSharedNotebooksRepository();
      await _pump(tester, books: books);
      await _tapText(tester, 'Share');
      expect(
        find.text('Type the username of the person to invite.'),
        findsOneWidget,
      );
      expect(books.calls.where((c) => c.startsWith('share')), isEmpty);
    });

    testWidgets('says why a notebook can’t be shared, in the core’s words', (
      tester,
    ) async {
      final books = FixtureSharedNotebooksRepository()
        ..unshareable['book:1'] = 'Kanso can’t share the Inbox.';
      await _pump(tester, books: books);
      expect(find.text('Kanso can’t share the Inbox.'), findsOneWidget);
      expect(find.byType(BookInviteBar), findsNothing);
    });

    testWidgets('an empty notebook asks for a note first', (tester) async {
      final books = FixtureSharedNotebooksRepository()
        ..checks['book:1'] = const BookShareCheck(
          name: 'Studio',
          notes: 0,
          locked: 0,
          lockedName: 'Studio (locked)',
          leftOut: <String>[],
        );
      await _pump(tester, books: books);
      expect(
        find.text('This notebook has no notes to share yet. Write one first.'),
        findsOneWidget,
      );
      expect(find.byType(BookInviteBar), findsNothing);
    });

    testWidgets('signed out, it says what sharing needs', (tester) async {
      final books = FixtureSharedNotebooksRepository()..signedOut = true;
      await _pump(tester, books: books);
      expect(
        find.textContaining('It needs a Kanso account'),
        findsOneWidget,
      );
      expect(find.text('Sign in…'), findsOneWidget);
      expect(find.byType(BookInviteBar), findsNothing);
    });
  });

  group('ShareNotebookView for a shared notebook', () {
    testWidgets('lists its people and invites another one', (tester) async {
      final books = _ownedWithDan();
      final people = FixtureSharedNotesRepository()
        ..people['eve'] = PersonLookup.found(personRow('eve', verified: true));
      await _pump(tester, books: books, people: people);
      expect(find.text('Share “Studio”'), findsOneWidget);
      expect(find.text('People'), findsOneWidget);
      expect(find.text('ngoni'), findsOneWidget);
      expect(find.text('You'), findsOneWidget);
      expect(find.text('dan'), findsOneWidget);
      expect(find.text('Stop sharing'), findsOneWidget);

      await tester.enterText(find.byType(EditableText), 'eve');
      await _tapText(tester, 'Invite');
      expect(books.calls, contains('invite book:1 eve editor'));
      expect(
        find.text(
          'Invited eve. The notebook shows for them once they accept.',
        ),
        findsOneWidget,
      );
      expect(find.text('eve'), findsOneWidget);
    });

    testWidgets('changes a person’s role from their menu', (tester) async {
      final books = _ownedWithDan();
      await _pump(tester, books: books);
      // dan's role button opens the roles the core says may be given.
      await _tapText(tester, 'Can edit');
      expect(find.text('Manager'), findsOneWidget);
      expect(find.text('Remove from notebook…'), findsOneWidget);
      await _tapText(tester, 'Can comment');
      expect(books.calls, contains('setRole book:1 a_dan commenter'));
      expect(find.text('Can comment'), findsOneWidget);
    });

    testWidgets('the menu offers only what the core gives', (tester) async {
      final books = _ownedWithDan(
        myRole: BookRole.manager,
        roles: const <BookRole>[BookRole.editor, BookRole.viewer],
        removable: false,
      );
      await _pump(tester, books: books);
      await _tapText(tester, 'Can edit');
      Finder inMenu(String text) => find.descendant(
        of: find.byType(BookMemberMenu),
        matching: find.text(text),
      );
      expect(inMenu('Can view'), findsOneWidget);
      expect(inMenu('Manager'), findsNothing);
      expect(inMenu('Remove from notebook…'), findsNothing);
    });

    testWidgets('a person without a menu shows their role as words', (
      tester,
    ) async {
      final books = _ownedWithDan(
        myRole: BookRole.editor,
        roles: const <BookRole>[],
        removable: false,
      );
      await _pump(tester, books: books);
      expect(find.text('Can edit'), findsNWidgets(2));
      expect(find.byType(BookInviteBar), findsNothing);
      expect(find.text('Leave'), findsOneWidget);
    });

    testWidgets('removes someone after asking, in the core’s words', (
      tester,
    ) async {
      final books = _ownedWithDan();
      await _pump(tester, books: books);
      await _tapText(tester, 'Can edit');
      await _tapText(tester, 'Remove from notebook…');
      expect(find.text('Remove dan from "Studio"?'), findsOneWidget);
      expect(find.textContaining("changes the notebook's key"), findsOne);
      await _tapText(tester, 'Remove');
      expect(books.calls, contains('remove book:1 a_dan'));
      expect(
        find.text('Removed dan. The notebook has a new key.'),
        findsOneWidget,
      );
      expect(find.text('dan'), findsNothing);
    });

    testWidgets('Cancel on the remove question keeps them', (tester) async {
      final books = _ownedWithDan();
      await _pump(tester, books: books);
      await _tapText(tester, 'Can edit');
      await _tapText(tester, 'Remove from notebook…');
      await _tapText(tester, 'Cancel');
      expect(books.calls.where((c) => c.startsWith('remove')), isEmpty);
      expect(find.text('dan'), findsOneWidget);
    });

    testWidgets('the owner stops sharing; everyone keeps a copy', (
      tester,
    ) async {
      final books = _ownedWithDan();
      await _pump(tester, books: books);
      await _tapText(tester, 'Stop sharing');
      expect(
        find.textContaining('Stop sharing "Studio"? Everyone keeps'),
        findsOneWidget,
      );
      await _tapText(tester, 'Stop sharing');
      expect(books.calls, contains('stop book:1'));
      expect(
        find.text(
          '"Studio" isn\'t shared any more. Everyone keeps a private '
          'copy.',
        ),
        findsOneWidget,
      );
      expect(find.text('People'), findsNothing);
    });

    testWidgets('a member leaves and keeps a private copy', (tester) async {
      final books = _ownedWithDan(
        myRole: BookRole.editor,
        roles: const <BookRole>[],
        removable: false,
      );
      await _pump(tester, books: books);
      await _tapText(tester, 'Leave');
      expect(
        find.text('Leave "Studio"? You\'ll keep a private copy.'),
        findsOneWidget,
      );
      await _tapText(tester, 'Leave');
      expect(books.calls, contains('leave book:1'));
      expect(
        find.text(
          'You left "Studio". Your copy is in "Studio (no longer shared)".',
        ),
        findsOneWidget,
      );
    });

    testWidgets('says why a change failed, and keeps the people', (
      tester,
    ) async {
      final books = _ownedWithDan()
        ..failure = const ApplicationError.network(
          message: 'Kanso can’t reach your sync server.',
        );
      await _pump(tester, books: books);
      await _tapText(tester, 'Can edit');
      await _tapText(tester, 'Can view');
      expect(books.calls, contains('setRole book:1 a_dan viewer'));
      expect(find.text('dan'), findsOneWidget);
      // dan keeps the role he had.
      expect(find.widgetWithText(KansoButton, 'Can edit'), findsOneWidget);
      expect(find.widgetWithText(KansoButton, 'Can view'), findsNothing);
    });
  });
}
