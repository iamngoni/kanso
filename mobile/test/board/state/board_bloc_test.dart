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
      boards.moved = CardMoved.missing;
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
  });
}
