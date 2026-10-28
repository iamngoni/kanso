// Kanso — fixture_board_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/board_repository.dart';

import 'library_fixtures.dart';

/// Boards over the mock notes, for widget and golden tests: the cards are
/// fields, a move really moves a card, and every move is recorded.
class FixtureBoardRepository implements BoardRepository {
  FixtureBoardRepository() {
    noteCards = <String, List<BoardCard>>{
      'none': <BoardCard>[
        const BoardCard(
          key: CardKey.note(id: MockIds.weeklyReview),
          title: 'Weekly review',
          noteTitle: '',
          snippet:
              'Finished the sync conflict screen and moved the Garden '
              'notes to next week.',
          checked: false,
          locked: false,
        ),
        const BoardCard(
          key: CardKey.note(id: MockIds.privateJournal),
          title: 'Private journal',
          noteTitle: '',
          snippet: '',
          checked: false,
          locked: true,
        ),
      ],
      'active': <BoardCard>[
        const BoardCard(
          key: CardKey.note(id: MockIds.syncDesign),
          title: 'Sync design notes',
          noteTitle: '',
          snippet:
              'Every device keeps a full copy. A change is encrypted '
              'before it leaves the device.',
          checked: false,
          locked: false,
        ),
      ],
      'onHold': <BoardCard>[
        const BoardCard(
          key: CardKey.note(id: MockIds.lisbon),
          title: 'Lisbon trip',
          noteTitle: '',
          snippet: 'Flights, the apartment and a list of places to eat.',
          checked: false,
          locked: false,
        ),
      ],
      'completed': <BoardCard>[],
      'dropped': <BoardCard>[],
    };
    todo = <BoardCard>[
      const BoardCard(
        key: CardKey.task(
          noteId: MockIds.weeklyReview,
          line: 4,
          text: 'Call about the allotment',
        ),
        title: 'Call about the allotment',
        noteTitle: 'Weekly review',
        snippet: '',
        due: '2026-10-27T09:00',
        dueChip: DueChipRow(
          label: 'Tomorrow 09:00',
          tone: DueTone.plain,
          tooltip: 'Due Tuesday, 27 October 2026 at 09:00',
        ),
        checked: false,
        locked: false,
      ),
      const BoardCard(
        key: CardKey.task(
          noteId: MockIds.syncDesign,
          line: 12,
          text: 'Write the conflict copy',
        ),
        title: 'Write the conflict copy',
        noteTitle: 'Sync design notes',
        snippet: '',
        checked: false,
        locked: false,
      ),
    ];
    done = <BoardCard>[
      const BoardCard(
        key: CardKey.task(
          noteId: MockIds.weeklyReview,
          line: 5,
          text: 'Plan the beds',
        ),
        title: 'Plan the beds',
        noteTitle: 'Weekly review',
        snippet: '',
        due: '2026-10-25',
        dueChip: DueChipRow(
          label: 'Yesterday',
          tone: DueTone.done,
          tooltip: 'Due Sunday, 25 October 2026',
        ),
        checked: true,
        locked: false,
      ),
    ];
  }

  late Map<String, List<BoardCard>> noteCards;
  late List<BoardCard> todo;
  late List<BoardCard> done;

  /// When set, `board` fails with it.
  ApplicationError? boardFailure;

  /// When set, `moveCard` fails with it.
  ApplicationError? moveFailure;

  /// What a move answers when it works.
  CardMoved moved = CardMoved.changed;

  /// When set, `board` waits for it.
  Completer<void>? gate;

  /// Every empty board.
  bool empty = false;

  final List<(BoardKind, ListQuery)> boards = <(BoardKind, ListQuery)>[];
  final List<(CardKey, BoardLane)> moves = <(CardKey, BoardLane)>[];

  @override
  Future<Either<ApplicationError, BoardView>> board(
    BoardKind kind,
    ListQuery query,
  ) async {
    boards.add((kind, query));
    await gate?.future;
    final error = boardFailure;
    if (error != null) {
      return Left<ApplicationError, BoardView>(error);
    }
    List<BoardCard> cards(List<BoardCard> source) =>
        empty ? const <BoardCard>[] : List<BoardCard>.of(source);
    return Right<ApplicationError, BoardView>(
      kind == BoardKind.notes
          ? BoardView(
              kind: kind,
              columns: <BoardColumn>[
                BoardColumn(
                  lane: const BoardLane.status(status: 'none'),
                  label: 'No status',
                  cards: cards(noteCards['none']!),
                ),
                BoardColumn(
                  lane: const BoardLane.status(status: 'active'),
                  label: 'Active',
                  cards: cards(noteCards['active']!),
                ),
                BoardColumn(
                  lane: const BoardLane.status(status: 'onHold'),
                  label: 'On hold',
                  cards: cards(noteCards['onHold']!),
                ),
                BoardColumn(
                  lane: const BoardLane.status(status: 'completed'),
                  label: 'Completed',
                  cards: cards(noteCards['completed']!),
                ),
                BoardColumn(
                  lane: const BoardLane.status(status: 'dropped'),
                  label: 'Dropped',
                  cards: cards(noteCards['dropped']!),
                ),
              ],
            )
          : BoardView(
              kind: kind,
              columns: <BoardColumn>[
                BoardColumn(
                  lane: const BoardLane.todo(),
                  label: 'To do',
                  cards: cards(todo),
                ),
                BoardColumn(
                  lane: const BoardLane.done(),
                  label: 'Done',
                  cards: cards(done),
                ),
              ],
            ),
    );
  }

  @override
  Future<Either<ApplicationError, CardMoved>> moveCard(
    CardKey card,
    BoardLane lane,
    int nowMs,
  ) async {
    moves.add((card, lane));
    final error = moveFailure;
    if (error != null) {
      return Left<ApplicationError, CardMoved>(error);
    }
    if (moved != CardMoved.changed) {
      return Right<ApplicationError, CardMoved>(moved);
    }
    BoardCard? found;
    for (final list in <List<BoardCard>>[
      ...noteCards.values,
      todo,
      done,
    ]) {
      final index = list.indexWhere((it) => it.key == card);
      if (index >= 0) {
        found = list.removeAt(index);
      }
    }
    if (found != null) {
      switch (lane) {
        case BoardLane_Status(:final status):
          noteCards[status]!.add(found);
        case BoardLane_Todo():
          todo.add(found.copyWithChecked(checked: false));
        case BoardLane_Done():
          done.add(found.copyWithChecked(checked: true));
      }
    }
    return const Right<ApplicationError, CardMoved>(CardMoved.changed);
  }
}

extension on BoardCard {
  BoardCard copyWithChecked({required bool checked}) => BoardCard(
    key: key,
    title: title,
    noteTitle: noteTitle,
    snippet: snippet,
    due: due,
    dueChip: dueChip,
    checked: checked,
    locked: locked,
  );
}
