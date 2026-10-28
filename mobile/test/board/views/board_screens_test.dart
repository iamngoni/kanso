// Kanso — board_screens_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/board/views/widgets/board_pane.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/note/views/pages/note_page.dart';
import 'package:kanso/tasks/views/widgets/task_due_chip.dart';

import '../../helpers/browse_blocs.dart';
import '../../helpers/library_fixtures.dart';
import '../../helpers/page_fixtures.dart';
import '../../helpers/pump_browse_app.dart';

const Size _phone = Size(393, 852);
const Size _ipad = Size(1180, 820);

Future<BrowseApp> _pump(
  WidgetTester tester,
  PageFixtures fixtures, {
  String at = '/notes/board',
  Size size = _phone,
}) => atMockNow(
  () => pumpBrowseApp(tester, size, at: at, fixtures: fixtures),
);

Finder _inBoard(String label) =>
    find.descendant(of: find.byType(BoardPane), matching: find.text(label));

void main() {
  group('the board', () {
    testWidgets('is reached from the list’s more menu', (tester) async {
      final fixtures = PageFixtures();
      final app = await _pump(tester, fixtures, at: '/notes');
      await tester.tap(find.bySemanticsLabel('More'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Board'));
      await tester.pumpAndSettle();
      expect(app.where, '/notes/board');
      expect(find.byType(BoardPane), findsOneWidget);
      expect(fixtures.boards.boards.single.$1, BoardKind.notes);

      await tester.tap(_inBoard('Tasks'));
      await tester.pumpAndSettle();
      expect(fixtures.boards.boards.last.$1, BoardKind.tasks);
      expect(find.text('To do'), findsOneWidget);
    });

    testWidgets('has no board for the trash', (tester) async {
      await _pump(tester, PageFixtures(), at: '/trash');
      await tester.tap(find.bySemanticsLabel('More'));
      await tester.pumpAndSettle();
      expect(find.text('Board'), findsNothing);
    });

    testWidgets('lays the notes out by status, with their words', (
      tester,
    ) async {
      final fixtures = PageFixtures();
      await _pump(tester, fixtures);
      expect(fixtures.boards.boards.single.$1, BoardKind.notes);
      expect(find.text('No status'), findsOneWidget);
      expect(find.text('Active'), findsOneWidget);
      expect(find.text('Weekly review'), findsOneWidget);
      expect(find.textContaining('Finished the sync conflict'), findsOneWidget);
      // A locked note shows its lock, not its text.
      expect(find.text('Private journal'), findsOneWidget);
      expect(find.text('Locked'), findsOneWidget);
      expect(
        find.text('Touch and hold a card to move it to another status.'),
        findsOneWidget,
      );
      expect(find.bySemanticsLabel('No status, 2 cards'), findsOneWidget);
    });

    testWidgets('the columns scroll sideways on a phone', (tester) async {
      await _pump(tester, PageFixtures());
      expect(find.text('Dropped'), findsNothing);
      await tester.dragUntilVisible(
        find.text('Dropped'),
        find.byType(Scrollable).first,
        const Offset(-200, 0),
      );
      expect(find.text('Dropped'), findsOneWidget);
    });

    testWidgets('switches between Notes and Tasks', (tester) async {
      final fixtures = PageFixtures();
      await _pump(tester, fixtures);
      await tester.tap(_inBoard('Tasks'));
      await tester.pumpAndSettle();
      expect(fixtures.boards.boards.last.$1, BoardKind.tasks);
      expect(find.text('To do'), findsOneWidget);
      expect(find.text('Done'), findsOneWidget);
      expect(find.text('Call about the allotment'), findsOneWidget);
      expect(find.text('Tomorrow 09:00'), findsOneWidget);
      expect(find.byType(TaskDueChip), findsNWidgets(2));
      expect(
        tester.widget<Text>(find.text('Yesterday')).style!.decoration,
        TextDecoration.lineThrough,
      );
      expect(
        find.text('Tick a task to do it, or touch and hold it to move it.'),
        findsOneWidget,
      );
    });

    testWidgets('a touch and hold asks where to move the card', (
      tester,
    ) async {
      final fixtures = PageFixtures();
      await _pump(tester, fixtures);
      await tester.longPress(find.text('Weekly review'));
      await tester.pumpAndSettle();
      expect(find.text('Move to'), findsOneWidget);
      await tester.tap(find.text('Completed').last);
      await tester.pumpAndSettle();
      expect(find.text('Move to'), findsNothing);
      expect(fixtures.boards.moves.single, (
        const CardKey.note(id: MockIds.weeklyReview),
        const BoardLane.status(status: 'completed'),
      ));
      // The board was read again and has it there.
      expect(fixtures.boards.noteCards['completed'], hasLength(1));
    });

    testWidgets('choosing the column it is in moves nothing', (tester) async {
      final fixtures = PageFixtures();
      await _pump(tester, fixtures);
      await tester.longPress(find.text('Weekly review'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('No status').last);
      await tester.pumpAndSettle();
      expect(fixtures.boards.moves, isEmpty);
    });

    testWidgets('ticking a task moves it to Done, and unticking back', (
      tester,
    ) async {
      final fixtures = PageFixtures();
      await _pump(tester, fixtures, at: '/notes/board?kind=tasks');
      await tester.tap(find.bySemanticsLabel('Mark done').first);
      await tester.pumpAndSettle();
      expect(fixtures.boards.moves.single.$2, const BoardLane.done());
      expect(fixtures.boards.done, hasLength(2));
      await tester.tap(find.bySemanticsLabel('Mark not done').first);
      await tester.pumpAndSettle();
      expect(fixtures.boards.moves.last.$2, const BoardLane.todo());
    });

    testWidgets('a tap opens the note, and back is the board', (tester) async {
      await _pump(tester, PageFixtures());
      await tester.tap(find.text('Weekly review'));
      await tester.pumpAndSettle();
      expect(find.byType(NotePage), findsOneWidget);
      expect(find.byType(BoardPane), findsNothing);
      await tester.tap(find.bySemanticsLabel('Back').first);
      await tester.pumpAndSettle();
      expect(find.byType(BoardPane), findsOneWidget);
    });

    testWidgets('a task card opens the note it is in', (tester) async {
      await _pump(tester, PageFixtures(), at: '/notes/board?kind=tasks');
      await tester.tap(find.text('Call about the allotment'));
      await tester.pumpAndSettle();
      final page = tester.widget<NotePage>(find.byType(NotePage));
      expect(page.noteId, MockIds.weeklyReview);
      expect(find.byType(BoardPane), findsNothing);
    });

    testWidgets('says when a move failed, and when a task moved', (
      tester,
    ) async {
      final fixtures = PageFixtures();
      await _pump(tester, fixtures, at: '/notes/board?kind=tasks');
      fixtures.boards.moveFailure = const ApplicationError.storage(
        message: 'Disk error.',
      );
      await tester.tap(find.bySemanticsLabel('Mark done').first);
      await tester.pumpAndSettle();
      expect(find.text('Couldn’t move the card.'), findsOneWidget);
      await tester.tap(find.text('OK'));
      await tester.pumpAndSettle();
      fixtures.boards.moveFailure = null;
      fixtures.boards.moved = CardMoved.missing;
      await tester.tap(find.bySemanticsLabel('Mark done').first);
      await tester.pumpAndSettle();
      expect(
        find.text(
          'That task changed in its note. The board is up to date now.',
        ),
        findsOneWidget,
      );
    });

    testWidgets('says what an empty board will hold', (tester) async {
      final fixtures = PageFixtures()..boards.empty = true;
      await _pump(tester, fixtures);
      expect(find.text('No notes here'), findsOneWidget);
      await tester.tap(_inBoard('Tasks'));
      await tester.pumpAndSettle();
      expect(find.text('No tasks in these notes'), findsOneWidget);
      expect(
        find.text('Write “- [ ] Call Sam” in a note, and it shows here.'),
        findsOneWidget,
      );
    });

    testWidgets('a board that cannot be read offers to try again', (
      tester,
    ) async {
      final fixtures = PageFixtures();
      fixtures.boards.boardFailure = const ApplicationError.storage(
        message: 'Disk error.',
      );
      await _pump(tester, fixtures);
      expect(find.text('The board didn’t load'), findsOneWidget);
      fixtures.boards.boardFailure = null;
      await tester.tap(find.text('Try again'));
      await tester.pumpAndSettle();
      expect(find.text('Weekly review'), findsOneWidget);
    });

    testWidgets('on a tablet cards are dragged, or moved by their button', (
      tester,
    ) async {
      final fixtures = PageFixtures();
      await _pump(tester, fixtures, size: _ipad);
      expect(find.text('Drag a card to change its status.'), findsOneWidget);

      // The pane is one column and a peek wide: bring On hold into view.
      final columns = find.byWidgetPredicate(
        (widget) =>
            widget is ListView && widget.scrollDirection == Axis.horizontal,
      );
      await tester.drag(columns, const Offset(-400, 0));
      await tester.pumpAndSettle();
      expect(_inBoard('On hold'), findsOneWidget);

      final gesture = await tester.startGesture(
        tester.getCenter(_inBoard('Sync design notes')),
      );
      await tester.pump(const Duration(milliseconds: 600));
      await gesture.moveTo(
        tester.getCenter(_inBoard('On hold')) + const Offset(0, 120),
      );
      await tester.pump();
      await gesture.up();
      await tester.pumpAndSettle();
      expect(
        fixtures.boards.moves.single.$2,
        const BoardLane.status(status: 'onHold'),
      );

      await tester.tap(find.bySemanticsLabel('Move to…').hitTestable().first);
      await tester.pumpAndSettle();
      expect(find.text('Move to'), findsOneWidget);
    });
  });
}
