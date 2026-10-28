// Kanso — boards_suite.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/library.dart';
import 'package:kanso/core/bridge/api/types.dart';

import 'open_library.dart';

/// The boards through the bridge: the notes a list shows by status, their
/// tasks by box, the moves that change one thing, and a due date's chip. It
/// runs on this computer (`test/core/bridge`) and on a simulator or an
/// emulator (`integration_test`).
void boardsSuite(OpenLibrary open) {
  group('KansoLibrary board', () {
    // 09:30 on 2026-05-20.
    const now = LocalTime(
      year: 2026,
      month: 5,
      day: 20,
      hour: 9,
      minute: 30,
      second: 0,
    );

    BoardColumn columnOf(BoardView board, BoardLane lane) =>
        board.columns.firstWhere((column) => column.lane == lane);

    test('notes stand in their status column, and a move changes it', () async {
      final library = await open();
      final made = await library.create(
        note: const NewNote(title: 'Plan', body: 'a plan'),
      );

      final board = await library.board(
        kind: BoardKind.notes,
        query: everyNote,
      );
      expect(board.kind, BoardKind.notes);
      expect(
        board.columns.map((column) => column.label),
        <String>['No status', 'Active', 'On hold', 'Completed', 'Dropped'],
      );
      final unset = columnOf(board, const BoardLane.status(status: 'none'));
      expect(unset.cards.map((card) => card.title), <String>['Plan']);

      final moved = await library.moveCard(
        card: CardKey.note(id: made.id),
        to: const BoardLane.status(status: 'active'),
        nowMs: 5,
      );
      expect(moved, CardMoved.changed);
      expect((await library.note(id: made.id)).status, 'active');
      // Again, there is nothing to write.
      expect(
        await library.moveCard(
          card: CardKey.note(id: made.id),
          to: const BoardLane.status(status: 'active'),
          nowMs: 6,
        ),
        CardMoved.unchanged,
      );

      final after = await library.board(
        kind: BoardKind.notes,
        query: everyNote,
      );
      expect(
        columnOf(
          after,
          const BoardLane.status(status: 'active'),
        ).cards.single.title,
        'Plan',
      );
      // A note goes in a status column, and in one the library knows.
      await expectLater(
        library.moveCard(
          card: CardKey.note(id: made.id),
          to: const BoardLane.done(),
          nowMs: 7,
        ),
        throwsA(isA<KansoError_Invalid>()),
      );
      await expectLater(
        library.moveCard(
          card: CardKey.note(id: made.id),
          to: const BoardLane.status(status: 'nonsense'),
          nowMs: 8,
        ),
        throwsA(isA<KansoError_Invalid>()),
      );
      await expectLater(
        library.moveCard(
          card: const CardKey.note(id: 'note:nowhere'),
          to: const BoardLane.status(status: 'active'),
          nowMs: 9,
        ),
        throwsA(isA<KansoError_NotFound>()),
      );
    });

    test('tasks stand in To do and Done, and a move ticks one', () async {
      final library = await open();
      final made = await library.create(
        note: const NewNote(
          title: 'Chores',
          body: '- [ ] water the plants\n- [x] pay the rent\n',
        ),
      );

      final board = await library.board(
        kind: BoardKind.tasks,
        query: everyNote,
      );
      expect(board.kind, BoardKind.tasks);
      expect(
        board.columns.map((column) => column.label),
        <String>['To do', 'Done'],
      );
      final todo = columnOf(board, const BoardLane.todo());
      final done = columnOf(board, const BoardLane.done());
      expect(todo.cards.map((card) => card.title), <String>[
        'water the plants',
      ]);
      expect(done.cards.map((card) => card.title), <String>['pay the rent']);
      expect(todo.cards.single.noteTitle, 'Chores');

      final moved = await library.moveCard(
        card: CardKey.task(
          noteId: made.id,
          line: 0,
          text: 'water the plants',
        ),
        to: const BoardLane.done(),
        nowMs: 5,
      );
      expect(moved, CardMoved.changed);
      expect(
        (await library.note(id: made.id)).body,
        contains('- [x] water the plants'),
      );
    });

    test('a due date reads as a chip, or not at all', () {
      final today = KansoLibrary.dueChip(
        due: '2026-05-20T18:00',
        checked: false,
        now: now,
      );
      expect(today, isNotNull);
      expect(today!.label, 'Today 18:00');
      expect(today.tone, DueTone.today);
      expect(today.tooltip, startsWith('Due '));
      expect(today.tooltip, endsWith('at 18:00'));

      final done = KansoLibrary.dueChip(
        due: '2026-05-20T18:00',
        checked: true,
        now: now,
      );
      expect(done!.tone, DueTone.done);

      final overdue = KansoLibrary.dueChip(
        due: '2026-05-19',
        checked: false,
        now: now,
      );
      expect(overdue!.tone, DueTone.overdue);
      expect(overdue.label, 'Overdue · Yesterday');

      // Text that isn't a date has no chip.
      expect(
        KansoLibrary.dueChip(due: 'not a date', checked: false, now: now),
        isNull,
      );
    });
  });
}
