// Kanso — shared_notebooks_blocs_test.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:kanso/sharing/models/data/book_prompt.dart';
import 'package:kanso/sharing/models/data/share_problem.dart';
import 'package:kanso/sharing/models/data/share_sheet_status.dart';
import 'package:kanso/sharing/state/notebook_sharing/notebook_sharing_bloc.dart';
import 'package:kanso/sharing/state/share_notebook/share_notebook_bloc.dart';
import 'package:kanso/sharing/state/shared_notebooks/shared_notebooks_bloc.dart';

import '../../../helpers/fixture_shared_notebooks_repository.dart';
import '../../../helpers/fixture_shared_notes_repository.dart';
import '../../../helpers/settle.dart';
import '../../../helpers/sharing_fixtures.dart';

SharedEvent _event({
  List<String> notebooks = const <String>[],
  int bookInvitations = 0,
  List<String> bookNotices = const <String>[],
}) => SharedEvent(
  noteIds: const <String>[],
  invitations: 0,
  commentNotes: const <String>[],
  liveNotes: const <String>[],
  notebooks: notebooks,
  bookInvitations: bookInvitations,
  bookNotices: bookNotices,
);

const BookSyncStatus _upToDate = BookSyncStatus(
  offline: false,
  waiting: 0,
  live: false,
);

SharedBookRow _book(String id) => SharedBookRow(
  shareId: 's_$id',
  notebookId: id,
  name: 'Studio',
  myRole: BookRole.editor,
  owner: 'ana',
  members: const <String>['ana', 'ngoni'],
  status: _upToDate,
);

BookInvitationRow _invitation(String shareId) => BookInvitationRow(
  shareId: shareId,
  from: 'ana',
  role: BookRole.editor,
  invited: true,
  atMs: 0,
);

void main() {
  late FixtureSharedNotebooksRepository books;
  late FixtureSharedNotesRepository people;

  setUp(() {
    books = FixtureSharedNotebooksRepository();
    people = FixtureSharedNotesRepository();
  });

  group('SharedNotebooksBloc', () {
    test('reads the shared notebooks and the invitations', () async {
      books
        ..books = <SharedBookRow>[_book('book:1')]
        ..invitationList = <BookInvitationRow>[_invitation('s9')];
      final bloc = SharedNotebooksBloc(books, people)
        ..add(const SharedNotebooksEvent.started());
      addTearDown(bloc.close);
      await settle();
      expect(bloc.state.status, LoadStatus.ready);
      expect(bloc.state.books.map((b) => b.notebookId), <String>['book:1']);
      expect(bloc.state.invitations.map((i) => i.shareId), <String>['s9']);
    });

    test('signed out, it has nothing', () async {
      books
        ..books = <SharedBookRow>[_book('book:1')]
        ..signedOut = true;
      final bloc = SharedNotebooksBloc(books, people)
        ..add(const SharedNotebooksEvent.started());
      addTearDown(bloc.close);
      await settle();
      expect(bloc.state.status, LoadStatus.ready);
      expect(bloc.state.books, isEmpty);
      expect(bloc.state.invitations, isEmpty);
    });

    test('accepting joins and names the notebook to open', () async {
      books.invitationList = <BookInvitationRow>[_invitation('s9')];
      final bloc = SharedNotebooksBloc(books, people)
        ..add(const SharedNotebooksEvent.started());
      addTearDown(bloc.close);
      await settle();
      bloc.add(const SharedNotebooksEvent.accepted('s9'));
      await settle();
      expect(books.calls, contains('accept s9'));
      expect(bloc.state.joined, 'book:s9');
      expect(bloc.state.invitations, isEmpty);
      expect(bloc.state.working, isEmpty);
    });

    test('an accept that fails says why and keeps the invitation', () async {
      books
        ..invitationList = <BookInvitationRow>[_invitation('s9')]
        ..failure = const ApplicationError.network(message: 'offline');
      final bloc = SharedNotebooksBloc(books, people)
        ..add(const SharedNotebooksEvent.started());
      addTearDown(bloc.close);
      await settle();
      bloc.add(const SharedNotebooksEvent.accepted('s9'));
      await settle();
      expect(bloc.state.problem, isNotNull);
      expect(bloc.state.joined, isNull);
      expect(bloc.state.invitations, hasLength(1));
    });

    test('declining drops the invitation', () async {
      books.invitationList = <BookInvitationRow>[_invitation('s9')];
      final bloc = SharedNotebooksBloc(books, people)
        ..add(const SharedNotebooksEvent.started());
      addTearDown(bloc.close);
      await settle();
      bloc.add(const SharedNotebooksEvent.declined('s9'));
      await settle();
      expect(books.calls, contains('decline s9'));
      expect(bloc.state.invitations, isEmpty);
    });

    test('keeps each notice from a sync until it is read', () async {
      final bloc = SharedNotebooksBloc(books, people)
        ..add(const SharedNotebooksEvent.started());
      addTearDown(bloc.close);
      await settle();
      people.eventsController.add(
        _event(bookNotices: <String>['one', 'two']),
      );
      await settle();
      expect(bloc.state.notices, <String>['one', 'two']);
      bloc.add(const SharedNotebooksEvent.noticeRead());
      await settle();
      expect(bloc.state.notices, <String>['two']);
    });

    test('a sync that brings an invitation reads them again', () async {
      final bloc = SharedNotebooksBloc(books, people)
        ..add(const SharedNotebooksEvent.started());
      addTearDown(bloc.close);
      await settle();
      books.invitationList = <BookInvitationRow>[_invitation('s9')];
      people.eventsController.add(_event(bookInvitations: 1));
      await settle();
      expect(bloc.state.invitations, hasLength(1));
    });
  });

  group('NotebookSharingBloc', () {
    test('a notebook not shared has no status and no edits', () async {
      final bloc = NotebookSharingBloc(books, people)
        ..add(const NotebookSharingEvent.started('book:1'));
      addTearDown(bloc.close);
      await settle();
      expect(bloc.state.notebookId, 'book:1');
      expect(bloc.state.status, isNull);
      expect(bloc.state.edits, isEmpty);
    });

    test('reads the status and who edited each note', () async {
      books
        ..statuses['book:1'] = _upToDate
        ..editsOf['book:1'] = <NoteEditRow>[
          const NoteEditRow(noteId: 'n1', name: 'dan', me: false, atMs: 5),
        ];
      final bloc = NotebookSharingBloc(books, people)
        ..add(const NotebookSharingEvent.started('book:1'));
      addTearDown(bloc.close);
      await settle();
      expect(bloc.state.status?.offline, isFalse);
      expect(bloc.state.edits['n1']?.name, 'dan');
    });

    test('reads again only when a sync changed this notebook', () async {
      books.statuses['book:1'] = _upToDate;
      final bloc = NotebookSharingBloc(books, people)
        ..add(const NotebookSharingEvent.started('book:1'));
      addTearDown(bloc.close);
      await settle();
      books.statuses['book:1'] = const BookSyncStatus(
        offline: true,
        waiting: 2,
        live: false,
      );
      people.eventsController.add(_event(notebooks: <String>['book:2']));
      await settle();
      expect(bloc.state.status?.offline, isFalse);
      people.eventsController.add(_event(notebooks: <String>['book:1']));
      await settle();
      expect(bloc.state.status?.waiting, 2);
    });
  });

  group('ShareNotebookBloc', () {
    ShareNotebookBloc build() {
      final bloc = ShareNotebookBloc(books, people);
      addTearDown(bloc.close);
      return bloc;
    }

    test('a notebook not shared is checked first', () async {
      final bloc = build()..add(const ShareNotebookEvent.started('book:1'));
      await settle();
      expect(bloc.state.status, ShareSheetStatus.ready);
      expect(bloc.state.info, isNull);
      expect(bloc.state.check?.notes, 3);
      expect(books.calls, <String>['info book:1', 'canShare book:1']);
    });

    test('a shared one is read, with the invite role it may give', () async {
      books.shares['book:1'] = bookInfo(
        'book:1',
        myRole: BookRole.manager,
        inviteRoles: const <BookRole>[BookRole.commenter, BookRole.viewer],
      );
      final bloc = build()..add(const ShareNotebookEvent.started('book:1'));
      await settle();
      expect(bloc.state.info, isNotNull);
      // Editor isn't offered: the first role it may give is chosen.
      expect(bloc.state.inviteRole, BookRole.commenter);
    });

    test('signed out and refused notebooks say so', () async {
      books.signedOut = true;
      final out = build()..add(const ShareNotebookEvent.started('book:1'));
      await settle();
      expect(out.state.status, ShareSheetStatus.signedOut);

      books
        ..signedOut = false
        ..unshareable['book:2'] = 'No.';
      final refused = build()..add(const ShareNotebookEvent.started('book:2'));
      await settle();
      expect(refused.state.status, ShareSheetStatus.notShareable);
      expect(refused.state.reason, 'No.');
    });

    test('an empty name asks for one', () async {
      final bloc = build()..add(const ShareNotebookEvent.started('book:1'));
      await settle();
      bloc.add(const ShareNotebookEvent.invited('  '));
      await settle();
      expect(bloc.state.problem, const ShareProblem.nameMissing());
    });

    test('a verified person is shared with at once', () async {
      people.people['dan'] = PersonLookup.found(
        personRow('dan', verified: true),
      );
      final bloc = build()..add(const ShareNotebookEvent.started('book:1'));
      await settle();
      bloc.add(const ShareNotebookEvent.invited('dan'));
      await settle();
      expect(books.calls, contains('share book:1 dan editor'));
      expect(bloc.state.done, const BookDone.shared('dan'));
      expect(bloc.state.info?.members.map((m) => m.username), contains('dan'));
      expect(bloc.state.busy, isFalse);
    });

    test('a new person is asked about, then invited unverified', () async {
      people.people['bo'] = PersonLookup.found(personRow('bo'));
      final bloc = build()..add(const ShareNotebookEvent.started('book:1'));
      await settle();
      bloc.add(const ShareNotebookEvent.invited('bo'));
      await settle();
      expect(bloc.state.prompt, isA<BookPromptVerify>());
      bloc.add(const ShareNotebookEvent.invitedUnverified());
      await settle();
      expect(people.calls.where((c) => c.startsWith('verify')), isEmpty);
      expect(books.calls, contains('share book:1 bo editor'));
      expect(bloc.state.prompt, isNull);
    });

    test('changed keys are trusted, then compared, then invited', () async {
      books.shares['book:1'] = bookInfo('book:1');
      people.people['bo'] = PersonLookup.changed(
        pinned: personRow('bo', fingerprint: 'OLD'),
        offered: personRow('bo', fingerprint: 'NEW'),
      );
      final bloc = build()..add(const ShareNotebookEvent.started('book:1'));
      await settle();
      bloc.add(const ShareNotebookEvent.invited('bo'));
      await settle();
      expect(bloc.state.prompt, isA<BookPromptKeyChanged>());
      bloc.add(const ShareNotebookEvent.newKeyAccepted());
      await settle();
      expect(bloc.state.prompt, isA<BookPromptVerify>());
      bloc.add(const ShareNotebookEvent.fingerprintConfirmed());
      await settle();
      expect(books.calls, contains('invite book:1 bo editor'));
      expect(bloc.state.done, const BookDone.invited('bo'));
    });

    test('role changes and removals go to the core', () async {
      books.shares['book:1'] = bookInfo(
        'book:1',
        members: <BookMemberRow>[
          bookMember('ngoni', role: BookRole.owner, me: true),
          bookMember('dan', removable: true),
        ],
      );
      final bloc = build()..add(const ShareNotebookEvent.started('book:1'));
      await settle();
      bloc.add(
        const ShareNotebookEvent.memberRoleChanged('a_dan', BookRole.viewer),
      );
      await settle();
      expect(books.calls, contains('setRole book:1 a_dan viewer'));
      final dan = bloc.state.info!.members.last;
      expect(dan.role, BookRole.viewer);
      bloc.add(ShareNotebookEvent.memberRemoved(dan));
      await settle();
      expect(books.calls, contains('remove book:1 a_dan'));
      expect(bloc.state.done, const BookDone.removed('dan'));
      expect(bloc.state.info!.members, hasLength(1));
    });

    test('the owner stops; a member leaves', () async {
      books.shares['book:1'] = bookInfo('book:1');
      books.shares['book:2'] = bookInfo('book:2', myRole: BookRole.editor);
      final owner = build()..add(const ShareNotebookEvent.started('book:1'));
      final member = build()..add(const ShareNotebookEvent.started('book:2'));
      await settle();
      owner.add(const ShareNotebookEvent.ended());
      member.add(const ShareNotebookEvent.ended());
      await settle();
      expect(books.calls, containsAll(<String>['stop book:1', 'leave book:2']));
      expect(owner.state.done, isA<BookDoneEnded>());
      expect(owner.state.info, isNull);
      expect(member.state.done, isA<BookDoneEnded>());
    });

    test('progress shows only for this notebook while sharing', () async {
      final bloc = build()..add(const ShareNotebookEvent.started('book:1'));
      await settle();
      bloc.add(
        const ShareNotebookEvent.progressed(
          BookShareProgress(notebookId: 'book:1', done: 1, total: 2),
        ),
      );
      await settle();
      // Not busy: nothing is going up.
      expect(bloc.state.progress, isNull);
    });
  });
}
