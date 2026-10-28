// Kanso — comments_bloc_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados, IntAnys, ListAnys, any;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:kanso/sharing/state/comments/comments_bloc.dart';

import '../../../helpers/fixture_shared_notes_repository.dart';
import '../../../helpers/property.dart';
import '../../../helpers/sharing_fixtures.dart';

const String _note = 'note:plan';

Future<CommentsBloc> _started(FixtureSharedNotesRepository shared) async {
  final bloc = CommentsBloc(shared)..add(const CommentsEvent.started(_note));
  await bloc.stream.firstWhere((state) => state.status != LoadStatus.loading);
  addTearDown(bloc.close);
  return bloc;
}

FixtureSharedNotesRepository _sharedWith(
  List<CommentRow> comments, {
  ShareRole role = ShareRole.editor,
}) => FixtureSharedNotesRepository()
  ..shares[_note] = shareInfo(_note, myRole: role)
  ..commentLists[_note] = comments;

void main() {
  group('CommentsBloc', () {
    test('a note that isn’t shared has no comments, and says so', () async {
      final bloc = await _started(FixtureSharedNotesRepository());
      expect(bloc.state.status, LoadStatus.ready);
      expect(bloc.state.shared, isFalse);
      expect(bloc.state.comments, isEmpty);
    });

    test('reads the comments in the desktop’s order, the band and the '
        'role', () async {
      final shared = _sharedWith(<CommentRow>[
        commentRow('c1'),
        commentRow('c2', resolved: true),
        commentRow('c3', orphaned: true),
      ]);
      final bloc = await _started(shared);
      expect(bloc.state.shared, isTrue);
      expect(bloc.state.mayComment, isTrue);
      expect(bloc.state.open, 2);
      expect(bloc.state.band, '2 open · 1 resolved');
      expect(bloc.state.anchored.map((c) => c.id), <String>['c1', 'c2']);
      expect(bloc.state.orphaned.map((c) => c.id), <String>['c3']);
    });

    test('a viewer may not comment', () async {
      final bloc = await _started(
        _sharedWith(<CommentRow>[], role: ShareRole.viewer),
      );
      expect(bloc.state.shared, isTrue);
      expect(bloc.state.mayComment, isFalse);
    });

    test(
      'resolve, edit and delete change the comment and read again',
      () async {
        final shared = _sharedWith(<CommentRow>[commentRow('c1', mine: true)]);
        final bloc = await _started(shared);
        bloc.add(const CommentsEvent.resolved(commentId: 'c1', resolved: true));
        await bloc.stream.firstWhere((s) => s.comments.single.resolved);
        bloc.add(const CommentsEvent.edited(commentId: 'c1', body: 'Better.'));
        await bloc.stream.firstWhere(
          (s) => s.comments.single.body == 'Better.',
        );
        bloc.add(const CommentsEvent.deleted('c1'));
        await bloc.stream.firstWhere((s) => s.comments.isEmpty);
        expect(
          shared.calls,
          containsAllInOrder(<String>[
            'resolveComment $_note c1 true',
            'editComment $_note c1 Better.',
            'deleteComment $_note c1',
          ]),
        );
      },
    );

    test('a change that fails says why and keeps the list', () async {
      final shared = _sharedWith(<CommentRow>[commentRow('c1', mine: true)]);
      final bloc = await _started(shared);
      shared.commentFailure = const ApplicationError.invalid(
        message: 'You can only change your own comments.',
      );
      bloc.add(const CommentsEvent.deleted('c1'));
      final failed = await bloc.stream.firstWhere((s) => s.error != null);
      expect(failed.error?.message, 'You can only change your own comments.');
      expect(failed.comments.single.id, 'c1');
    });

    test(
      'a sync that says the note’s comments changed reads them again',
      () async {
        final shared = _sharedWith(<CommentRow>[]);
        final bloc = await _started(shared);
        shared.commentLists[_note] = <CommentRow>[commentRow('new')];
        shared.eventsController.add(
          const SharedEvent(
            noteIds: <String>[],
            invitations: 0,
            commentNotes: <String>[_note],
            liveNotes: <String>[],
          ),
        );
        final now = await bloc.stream.firstWhere((s) => s.comments.isNotEmpty);
        expect(now.comments.single.id, 'new');
      },
    );

    test('news of other notes is not read', () async {
      final shared = _sharedWith(<CommentRow>[]);
      final bloc = await _started(shared);
      final reads = shared.calls.where((c) => c.startsWith('comments ')).length;
      shared.eventsController.add(
        const SharedEvent(
          noteIds: <String>['note:other'],
          invitations: 0,
          commentNotes: <String>['note:other'],
          liveNotes: <String>[],
        ),
      );
      await pumpEventQueue();
      expect(
        shared.calls.where((c) => c.startsWith('comments ')).length,
        reads,
      );
      expect(bloc.state.comments, isEmpty);
    });

    test(
      'a selection waits for the editor, which says when it showed it',
      () async {
        final bloc = await _started(
          _sharedWith(<CommentRow>[commentRow('c1')]),
        );
        bloc.add(const CommentsEvent.selected('c1'));
        await bloc.stream.firstWhere((s) => s.selecting == 'c1');
        bloc.add(const CommentsEvent.selectionHandled());
        await bloc.stream.firstWhere((s) => s.selecting == null);
      },
    );

    Glados<List<int>>(
      any.listWithLengthInRange(0, 10, any.intInRange(0, 2)),
      kansoExploreConfig(),
    ).test('the count of open comments is the list’s, after any resolving', (
      flags,
    ) async {
      final shared = _sharedWith(<CommentRow>[
        for (var i = 0; i < flags.length; i++)
          commentRow('c$i', start: i, end: i + 1, resolved: flags[i] == 1),
      ]);
      final bloc = CommentsBloc(shared)
        ..add(const CommentsEvent.started(_note));
      await pumpEventQueue();
      expect(bloc.state.open, flags.where((f) => f == 0).length);
      expect(
        bloc.state.anchored.length + bloc.state.orphaned.length,
        flags.length,
      );
      await bloc.close();
    });
  });
}
