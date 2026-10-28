// Kanso — shared_suite.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';

import 'open_library.dart';

/// Shared notes, through the bridge: every call of `KansoShared`, with no
/// account and no server.
void sharedSuite(OpenLibrary open) {
  group('KansoShared', () {
    test('a note that is not there cannot be shared or changed', () async {
      final shared = (await open()).shared();
      const note = 'no-such-note';
      await expectLater(
        shared.canShare(noteId: note),
        throwsA(isA<KansoError_NotFound>()),
      );
      await expectLater(
        shared.share(
          noteId: note,
          invites: <(String, ShareRole)>[('bob', ShareRole.editor)],
        ),
        throwsA(isA<KansoError_NotFound>()),
      );
      await expectLater(
        shared.info(noteId: note),
        throwsA(isA<KansoError_NotFound>()),
      );
      await expectLater(
        shared.invite(
          noteId: note,
          username: 'bob',
          role: ShareRole.viewer,
        ),
        throwsA(isA<KansoError_NotFound>()),
      );
      await expectLater(
        shared.setRole(
          noteId: note,
          accountId: 'bob',
          role: ShareRole.viewer,
        ),
        throwsA(isA<KansoError_NotFound>()),
      );
      await expectLater(
        shared.remove(noteId: note, accountId: 'bob'),
        throwsA(isA<KansoError_NotFound>()),
      );
    });

    test(
      'a note that is not there has no requests, links or live room',
      () async {
        final shared = (await open()).shared();
        const note = 'no-such-note';
        await expectLater(
          shared.approveRequest(noteId: note, accountId: 'bob'),
          throwsA(isA<KansoError_NotFound>()),
        );
        await expectLater(
          shared.declineRequest(noteId: note, accountId: 'bob'),
          throwsA(isA<KansoError_NotFound>()),
        );
        await expectLater(
          shared.createLink(noteId: note, role: ShareRole.viewer, days: 1),
          throwsA(isA<KansoError_NotFound>()),
        );
        await expectLater(
          shared.cancelLink(noteId: note, linkId: 'link'),
          throwsA(isA<KansoError_NotFound>()),
        );
        await expectLater(
          shared.stop(noteId: note),
          throwsA(isA<KansoError_NotFound>()),
        );
        await expectLater(
          shared.leave(noteId: note),
          throwsA(isA<KansoError_NotFound>()),
        );
        await expectLater(
          shared.openLive(noteId: note),
          throwsA(isA<KansoError_NotFound>()),
        );
        await shared.closeLive(noteId: note);
      },
    );

    test('a real note is not shared with no account behind it', () async {
      final library = await open();
      final id = await makeNote(library, 'Plan', 'Body');
      await expectLater(
        library.shared().info(noteId: id),
        throwsA(isA<KansoError>()),
      );
      expect((await library.note(id: id)).body, 'Body');
    });

    test('signed out, the invitations and people are out of reach', () async {
      final shared = (await open()).shared();
      await expectLater(
        shared.join(link: 'https://example.invalid/join'),
        throwsA(isA<KansoError_SignedOut>()),
      );
      await expectLater(
        shared.invitations(),
        throwsA(isA<KansoError_SignedOut>()),
      );
      await expectLater(
        shared.accept(shareId: 'share'),
        throwsA(isA<KansoError_SignedOut>()),
      );
      await expectLater(
        shared.decline(shareId: 'share'),
        throwsA(isA<KansoError_SignedOut>()),
      );
      await expectLater(
        shared.lookupPerson(username: 'bob'),
        throwsA(isA<KansoError_SignedOut>()),
      );
      await expectLater(
        shared.verifyPerson(accountId: 'bob'),
        throwsA(isA<KansoError_SignedOut>()),
      );
      await expectLater(
        shared.acceptNewKey(username: 'bob', fingerprint: 'abcd'),
        throwsA(isA<KansoError_SignedOut>()),
      );
      await expectLater(
        shared.syncShared(),
        throwsA(isA<KansoError_SignedOut>()),
      );
    });

    test('a note shared with nobody has no comments to read or write, and '
        'signed out they are out of reach', () async {
      final library = await open();
      final shared = library.shared();
      final id = await makeNote(library, 'Plan', 'Body');
      // Read with no account behind them: refused, as everything shared is.
      await expectLater(
        shared.comments(noteId: id),
        throwsA(isA<KansoError>()),
      );
      await expectLater(
        shared.openComments(noteId: id),
        throwsA(isA<KansoError>()),
      );
      await expectLater(
        shared.commentsBand(noteId: id),
        throwsA(isA<KansoError>()),
      );
      await expectLater(
        shared.addComment(noteId: id, start: 0, end: 4, body: 'Why?'),
        throwsA(isA<KansoError>()),
      );
      await expectLater(
        shared.editComment(noteId: id, commentId: 'c', body: 'Why not?'),
        throwsA(isA<KansoError>()),
      );
      await expectLater(
        shared.resolveComment(noteId: id, commentId: 'c', resolved: true),
        throwsA(isA<KansoError>()),
      );
      await expectLater(
        shared.deleteComment(noteId: id, commentId: 'c'),
        throwsA(isA<KansoError>()),
      );
      // A note that is not there is not found first.
      await expectLater(
        shared.openComments(noteId: 'no-such-note'),
        throwsA(isA<KansoError_NotFound>()),
      );
      expect((await library.note(id: id)).body, 'Body');
    });

    test('a note shared with nobody has no live status and no problems to '
        'dismiss', () async {
      final library = await open();
      final shared = library.shared();
      final id = await makeNote(library, 'Plan', 'Body');
      expect(await shared.liveStatus(noteId: id), isNull);
      await expectLater(
        shared.dismissProblems(noteId: id),
        throwsA(isA<KansoError>()),
      );
      // A note that is not there is not found, as everywhere else.
      await expectLater(
        shared.liveStatus(noteId: 'no-such-note'),
        throwsA(isA<KansoError_NotFound>()),
      );
    });

    test('the events stream says nothing until something is found', () async {
      final shared = (await open()).shared();
      final events = <SharedEvent>[];
      final listening = shared.events().listen(events.add);
      await Future<void>.delayed(const Duration(milliseconds: 300));
      // Cancelling waits for the library's next event: don't wait for it.
      unawaited(listening.cancel());
      expect(events, isEmpty);
    });
  });
}
