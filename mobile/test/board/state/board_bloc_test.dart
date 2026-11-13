// Kanso — board_bloc_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/board/state/board/board_bloc.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/load_status.dart';

import '../../helpers/fixture_board_repository.dart';
import '../../helpers/fixture_library_repository.dart';
import '../../helpers/library_fixtures.dart';

const _query = ListQuery(
  place: ListPlace.all(),
  pinnedOnly: false,
  sort: SortKey.updated,
  descending: true,
);

const _task = CardKey.task(
  noteId: MockIds.syncDesign,
  line: 12,
  text: 'Write the conflict copy',
);

void main() {
  late FixtureBoardRepository boards;
  late FixtureLibraryRepository library;

  setUp(() {
    boards = FixtureBoardRepository();
    library = FixtureLibraryRepository();
    addTearDown(library.close);
  });

  Future<BoardBloc> started([BoardKind kind = BoardKind.notes]) async {
    final bloc = BoardBloc(boards, library)
      ..add(BoardEvent.started(query: _query, kind: kind));
    addTearDown(bloc.close);
    await bloc.stream.firstWhere((s) => s.status == LoadStatus.ready);
    return bloc;
  }

  group('BoardBloc', () {
    test('reads the notes board of the query', () async {
      final bloc = await started();
      expect(boards.boards.single, (BoardKind.notes, _query));
      expect(bloc.state.data!.columns.map((c) => c.label), <String>[
        'No status',
        'Active',
        'On hold',
        'Completed',
        'Dropped',
      ]);
    });

    test('switching to tasks reads the tasks board', () async {
      final bloc = await started();
      bloc.add(const BoardEvent.kindChanged(BoardKind.tasks));
      final state = await bloc.stream.firstWhere(
        (s) => s.kind == BoardKind.tasks && s.status == LoadStatus.ready,
      );
      expect(state.data!.columns.map((c) => c.label), <String>[
        'To do',
        'Done',
      ]);
    });

    test('a move is written, then the board is read again', () async {
      final bloc = await started(BoardKind.tasks);
      bloc.add(
        const BoardEvent.cardMoved(card: _task, lane: BoardLane.done()),
      );
      final state = await bloc.stream.firstWhere(
        (s) => s.data!.columns.last.cards.length == 2,
      );
      expect(boards.moves.single, (_task, const BoardLane.done()));
      expect(state.notice, isNull);
    });

    test('a failed move says so and reads the board again', () async {
      final bloc = await started(BoardKind.tasks);
      boards.moveFailure = const ApplicationError.storage(message: 'Disk.');
      final readings = boards.boards.length;
      bloc.add(
        const BoardEvent.cardMoved(card: _task, lane: BoardLane.done()),
      );
      final state = await bloc.stream.firstWhere((s) => s.notice != null);
      expect(state.notice, isA<BoardNoticeFailed>());
      await Future<void>.delayed(Duration.zero);
      expect(boards.boards.length, greaterThan(readings));
      bloc.add(const BoardEvent.noticeDismissed());
      await bloc.stream.firstWhere((s) => s.notice == null);
    });

    test('a task that changed in its note is told so', () async {
      final bloc = await started(BoardKind.tasks);
      boards.moved = const CardMoved.missing();
      bloc.add(
        const BoardEvent.cardMoved(card: _task, lane: BoardLane.done()),
      );
      final state = await bloc.stream.firstWhere((s) => s.notice != null);
      expect(state.notice, isA<BoardNoticeChangedInNote>());
    });

    test('a change to the library reads the board again', () async {
      await started();
      final readings = boards.boards.length;
      library.changeStream.add(<String>[MockIds.syncDesign]);
      await Future<void>.delayed(const Duration(milliseconds: 300));
      expect(boards.boards.length, readings + 1);
    });

    test('a board that cannot be read is a failure; retry reads it', () async {
      boards.boardFailure = const ApplicationError.storage(message: 'Disk.');
      final bloc = BoardBloc(boards, library)
        ..add(const BoardEvent.started(query: _query));
      addTearDown(bloc.close);
      await bloc.stream.firstWhere((s) => s.status == LoadStatus.failed);
      boards.boardFailure = null;
      bloc.add(const BoardEvent.retried());
      await bloc.stream.firstWhere((s) => s.status == LoadStatus.ready);
    });

    group('the settings of a notebook board', () {
      const notebook = ListQuery(
        place: ListPlace.notebook(id: 'book:studio'),
        pinnedOnly: false,
        sort: SortKey.updated,
        descending: true,
      );

      Future<BoardBloc> studio() async {
        boards.studio = true;
        final bloc = BoardBloc(boards, library)
          ..add(const BoardEvent.started(query: notebook));
        addTearDown(bloc.close);
        await bloc.stream.firstWhere((s) => s.status == LoadStatus.ready);
        return bloc;
      }

      test('read the lanes and what the board is grouped by', () async {
        final bloc = await studio();
        final setup = bloc.state.data!.setup!;
        expect(setup.bookId, 'book:studio');
        expect(setup.group, 'stage');
        expect(setup.lanes, <String>['Idea', 'Scripting', 'Shooting']);
        expect(
          bloc.state.data!.columns.map((c) => c.label),
          <String>['Idea', 'Scripting', 'Shooting', 'No stage'],
        );
      });

      test('a status board has none', () async {
        final bloc = await started();
        expect(bloc.state.data!.setup, isNull);
        bloc.add(const BoardEvent.groupChanged('stage'));
        await Future<void>.delayed(Duration.zero);
        expect(boards.settingsCalls, isEmpty);
      });

      test('grouping and the empty lane are written, then read', () async {
        final bloc = await studio();
        bloc.add(const BoardEvent.groupChanged(null));
        final state = await bloc.stream.firstWhere(
          (s) => s.data?.setup?.group == null,
        );
        expect(
          state.data!.columns.first.lane,
          const BoardLane.status(status: 'none'),
        );
        bloc.add(const BoardEvent.groupChanged('stage'));
        await bloc.stream.firstWhere((s) => s.data?.setup?.group == 'stage');
        bloc.add(const BoardEvent.showUnsetChanged(show: false));
        await bloc.stream.firstWhere((s) => s.data?.setup?.showUnset == false);
        expect(boards.settingsCalls, <String>[
          'group -',
          'group stage',
          'show false',
        ]);
      });

      test('an edit says what it did, with a ticket to undo it', () async {
        final bloc = await studio();
        bloc.add(
          const BoardEvent.laneEdited(LaneChange.add(name: 'Editing')),
        );
        final state = await bloc.stream.firstWhere((s) => s.notice != null);
        expect(
          state.notice,
          const BoardNotice.done(message: 'Added the lane Editing.', undo: 9),
        );
        await bloc.stream.firstWhere(
          (s) => s.data?.setup?.lanes.length == 4,
        );
        bloc.add(const BoardEvent.undone(9));
        await bloc.stream.firstWhere((s) => s.notice == null);
        await Future<void>.delayed(Duration.zero);
        expect(library.undone, <int>[9]);
      });

      test('a refused edit keeps the core’s words and no notice', () async {
        final bloc = await studio();
        boards.refusal = 'A lane needs a name.';
        bloc.add(const BoardEvent.laneEdited(LaneChange.add(name: ' ')));
        final state = await bloc.stream.firstWhere(
          (s) => s.settingsError != null,
        );
        expect(state.settingsError, 'A lane needs a name.');
        expect(state.notice, isNull);
        bloc.add(const BoardEvent.settingsClosed());
        await bloc.stream.firstWhere((s) => s.settingsError == null);
      });

      test('removing an empty lane goes at once', () async {
        final bloc = await studio();
        boards.laneCards['Scripting'] = <BoardCard>[];
        bloc.add(const BoardEvent.laneRemovalRequested('Scripting'));
        await bloc.stream.firstWhere((s) => s.notice != null);
        expect(boards.settingsCalls, <String>['remove Scripting>-']);
        expect(bloc.state.removal, isNull);
      });

      test(
        'removing a lane with notes asks, and moves them where told',
        () async {
          final bloc = await studio();
          bloc.add(const BoardEvent.laneRemovalRequested('Scripting'));
          var state = await bloc.stream.firstWhere((s) => s.removal != null);
          expect(
            state.removal!.plan.prompt,
            '1 note is in Scripting. Move it to:',
          );
          // The next lane is chosen to begin with.
          expect(state.removal!.target, 'Shooting');
          expect(boards.settingsCalls, isEmpty);
          bloc.add(const BoardEvent.removalTargetChosen(null));
          state = await bloc.stream.firstWhere(
            (s) => s.removal?.target == null,
          );
          bloc.add(const BoardEvent.removalConfirmed());
          await bloc.stream.firstWhere(
            (s) => s.removal == null && s.notice != null,
          );
          expect(boards.settingsCalls, <String>['remove Scripting>-']);
        },
      );

      test('cancelling the question writes nothing', () async {
        final bloc = await studio();
        bloc.add(const BoardEvent.laneRemovalRequested('Scripting'));
        await bloc.stream.firstWhere((s) => s.removal != null);
        bloc.add(const BoardEvent.removalCancelled());
        await bloc.stream.firstWhere((s) => s.removal == null);
        expect(boards.settingsCalls, isEmpty);
      });

      test('a move to a lane says so and keeps a ticket', () async {
        final bloc = await studio();
        bloc.add(
          const BoardEvent.cardMoved(
            card: CardKey.note(id: MockIds.weeklyReview),
            lane: BoardLane.value(value: 'Shooting'),
          ),
        );
        final state = await bloc.stream.firstWhere((s) => s.notice != null);
        expect(
          state.notice,
          const BoardNotice.done(message: 'Moved to Shooting.', undo: 5),
        );
      });
    });
  });
}
