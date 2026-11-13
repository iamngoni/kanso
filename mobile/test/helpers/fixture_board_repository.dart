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
    laneCards = <String, List<BoardCard>>{
      'Idea': <BoardCard>[noteCards['none']!.first],
      'Scripting': <BoardCard>[noteCards['active']!.first],
      'Shooting': <BoardCard>[noteCards['onHold']!.first],
      '': <BoardCard>[noteCards['none']!.last],
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

  /// A notebook's board grouped by a property: the notes of a notebook
  /// place are laid out by [groupKey] in [lanes], as the core does, when
  /// [studio] is on. Cards are in [laneCards] by lane ('' is no value).
  bool studio = false;
  String? groupKey = 'stage';
  List<String> lanes = <String>['Idea', 'Scripting', 'Shooting'];
  bool showUnset = true;
  late Map<String, List<BoardCard>> laneCards;

  /// What the settings asked for, in order: 'group stage', 'show false',
  /// 'rename A>B', 'move 2>0', 'add X', 'remove A>B' (a null target is
  /// 'remove A>-').
  final List<String> settingsCalls = <String>[];

  /// What an edit answers when it is refused (the core's words).
  String? refusal;

  late Map<String, List<BoardCard>> noteCards;
  late List<BoardCard> todo;
  late List<BoardCard> done;

  /// When set, `board` fails with it.
  ApplicationError? boardFailure;

  /// When set, `moveCard` fails with it.
  ApplicationError? moveFailure;

  /// What a move answers when it works.
  CardMoved moved = const CardMoved.changed();

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
    if (studio &&
        kind == BoardKind.notes &&
        query.place is ListPlace_Notebook) {
      return Right<ApplicationError, BoardView>(_studioBoard(cards));
    }
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

  BoardView _studioBoard(List<BoardCard> Function(List<BoardCard>) cards) {
    final setup = BoardSetup(
      bookId: 'book:studio',
      group: groupKey,
      groupable: const <String>['stage'],
      lanes: groupKey == null ? const <String>[] : List<String>.of(lanes),
      showUnset: showUnset,
    );
    if (groupKey == null) {
      final status = <BoardColumn>[
        for (final entry in noteCards.entries)
          BoardColumn(
            lane: BoardLane.status(status: entry.key),
            label: entry.key,
            cards: cards(entry.value),
          ),
      ];
      return BoardView(
        kind: BoardKind.notes,
        columns: status,
        setup: setup,
      );
    }
    final unset = laneCards[''] ?? <BoardCard>[];
    return BoardView(
      kind: BoardKind.notes,
      setup: setup,
      columns: <BoardColumn>[
        for (final lane in lanes)
          BoardColumn(
            lane: BoardLane.value(value: lane),
            label: lane,
            cards: cards(laneCards[lane] ?? <BoardCard>[]),
          ),
        if (showUnset || unset.isNotEmpty)
          BoardColumn(
            lane: const BoardLane.unset(),
            label: 'No $groupKey',
            cards: cards(unset),
          ),
      ],
    );
  }

  @override
  Future<Either<ApplicationError, Unit>> setGroup(
    String bookId,
    String? by,
  ) async {
    settingsCalls.add('group ${by ?? '-'}');
    groupKey = by;
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, Unit>> setShowUnset(
    String bookId, {
    required bool show,
  }) async {
    settingsCalls.add('show $show');
    showUnset = show;
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, LaneRemoval>> laneRemoval(
    String bookId,
    String lane,
  ) async {
    final notes = (laneCards[lane] ?? <BoardCard>[]).length;
    final at = lanes.indexOf(lane);
    return Right<ApplicationError, LaneRemoval>(
      LaneRemoval(
        notes: notes,
        prompt: notes == 0
            ? ''
            : notes == 1
            ? '1 note is in $lane. Move it to:'
            : '$notes notes are in $lane. Move them to:',
        target: at + 1 < lanes.length ? lanes[at + 1] : lanes[at - 1],
        targets: <String>[
          for (final other in lanes)
            if (other != lane) other,
        ],
      ),
    );
  }

  @override
  Future<Either<ApplicationError, LaneEdited>> editLanes(
    String bookId,
    LaneChange change,
    int nowMs,
  ) async {
    final refused = refusal;
    if (refused != null) {
      return Left<ApplicationError, LaneEdited>(
        ApplicationError.invalid(message: refused),
      );
    }
    var message = '';
    var rewritten = 0;
    switch (change) {
      case LaneChange_Rename(:final from, :final to):
        settingsCalls.add('rename $from>$to');
        if (to != from && lanes.contains(to)) {
          return Left<ApplicationError, LaneEdited>(
            ApplicationError.invalid(
              message: 'There is a lane called $to already.',
            ),
          );
        }
        lanes[lanes.indexOf(from)] = to;
        laneCards[to] = laneCards.remove(from) ?? <BoardCard>[];
        rewritten = laneCards[to]!.length;
        message = 'Renamed $from to $to.';
      case LaneChange_Move(:final from, :final to):
        settingsCalls.add('move $from>$to');
        lanes.insert(to, lanes.removeAt(from));
        message = 'Moved a lane.';
      case LaneChange_Add(:final name):
        settingsCalls.add('add $name');
        lanes.add(name);
        message = 'Added the lane $name.';
      case LaneChange_Remove(:final lane, :final moveTo):
        settingsCalls.add('remove $lane>${moveTo ?? '-'}');
        final moved = laneCards.remove(lane) ?? <BoardCard>[];
        rewritten = moved.length;
        (laneCards[moveTo ?? ''] ??= <BoardCard>[]).addAll(moved);
        lanes.remove(lane);
        message = 'Removed $lane.';
    }
    return Right<ApplicationError, LaneEdited>(
      LaneEdited(
        lanes: List<String>.of(lanes),
        rewritten: rewritten,
        message: message,
        undo: 9,
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
    if (moved is! CardMoved_Changed) {
      return Right<ApplicationError, CardMoved>(moved);
    }
    if (lane is BoardLane_Value || lane is BoardLane_Unset) {
      BoardCard? taken;
      for (final list in laneCards.values) {
        final index = list.indexWhere((it) => it.key == card);
        if (index >= 0) {
          taken = list.removeAt(index);
        }
      }
      final to = switch (lane) {
        BoardLane_Value(:final value) => value,
        _ => '',
      };
      if (taken != null) {
        (laneCards[to] ??= <BoardCard>[]).add(taken);
      }
      return Right<ApplicationError, CardMoved>(
        CardMoved.changed(
          message: 'Moved to ${to.isEmpty ? 'No $groupKey' : to}.',
          undo: 5,
        ),
      );
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
        case BoardLane_Value() || BoardLane_Unset() || BoardLane_Unlisted():
          break;
      }
    }
    return const Right<ApplicationError, CardMoved>(CardMoved.changed());
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
