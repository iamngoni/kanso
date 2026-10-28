// Kanso — tasks_pane_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/browse_location.dart';
import 'package:kanso/core/state/undo_notice/undo_notice_bloc.dart';
import 'package:kanso/tasks/views/widgets/task_row.dart';
import 'package:kanso/tasks/views/widgets/tasks_pane.dart';

import '../../../helpers/browse_blocs.dart';
import '../../../helpers/fixture_library_repository.dart';
import '../../../helpers/library_fixtures.dart';
import '../../../helpers/pump_routed.dart';

Future<BrowseBlocs> _show(
  WidgetTester tester, {
  FixtureLibraryRepository? library,
  String? openNoteId,
  bool read = true,
}) async {
  final blocs = await BrowseBlocs.start(library);
  final tasks = await blocs.tasksOf(read: read);
  await pumpRouted(
    tester,
    blocs.provide(TasksPane(openNoteId: openNoteId), tasks: tasks),
  );
  await tester.pumpAndSettle();
  return blocs;
}

void main() {
  group('TasksPane', () {
    testWidgets('lists the groups under the desktop headers', (tester) async {
      await _show(tester);
      for (final label in <String>[
        'Overdue',
        'Today',
        'Upcoming',
        'No date',
        'Completed',
      ]) {
        // "Today" is a header, and the chip of two tasks.
        expect(find.text(label), findsWidgets);
      }
      expect(find.text('Send Sam the draft'), findsOneWidget);
      expect(find.text('Oct 20'), findsOneWidget);
      expect(find.text('Sync design · Projects'), findsOneWidget);
      expect(find.text('6 tasks'), findsOneWidget);
    });

    testWidgets('strikes a done task through and mutes it', (tester) async {
      await _show(tester);
      final text = tester.widget<Text>(find.text('Buy oat milk'));
      expect(text.style?.decoration, TextDecoration.lineThrough);
      final open = tester.widget<Text>(find.text('Send Sam the draft'));
      expect(open.style?.decoration, isNull);
    });

    testWidgets('ticks a task with its box: one write, the notice, a bump', (
      tester,
    ) async {
      final blocs = await _show(tester);
      // The box is at the start of the row.
      final row = tester.getTopLeft(find.byType(TaskRow).first);
      await tester.tapAt(row + const Offset(30, 30));
      await tester.pumpAndSettle();
      expect(blocs.library.tasksSet, <(String, int, String, bool)>[
        ('n-a', 4, 'Send Sam the draft @2026-10-20', true),
      ]);
      expect(blocs.undoNotice.state.notice, isNotNull);
      expect(blocs.undoNotice.state.notice?.message, 'Ticked off “Call Sam”');
      expect(blocs.haptics.selections, 1);
      // The list was read again with the task kept.
      expect(blocs.library.taskQueries.last.kept, <String>['a']);
    });

    testWidgets('opens the note of a task when its text is pressed', (
      tester,
    ) async {
      final blocs = await BrowseBlocs.start();
      final tasks = await blocs.tasksOf();
      final router = await pumpRouted(
        tester,
        blocs.provide(const TasksPane(), tasks: tasks),
      );
      await tester.tap(find.text('Book the venue'));
      await tester.pumpAndSettle();
      expect(
        whereIs(router),
        const BrowseLocation.tasks(noteId: 'n-b').location,
      );
      expect(blocs.history.state.pendingNoteId, isNull);
    });

    testWidgets('says a task moved, and Undo of the notice is the ticket', (
      tester,
    ) async {
      final library = FixtureLibraryRepository()
        ..setTaskResult = const Right<ApplicationError, Ticked>(
          Ticked.missing(),
        );
      await _show(tester, library: library);
      final row = tester.getTopLeft(find.byType(TaskRow).first);
      await tester.tapAt(row + const Offset(30, 30));
      await tester.pumpAndSettle();
      expect(
        find.text('That task changed in its note. The list is up to date now.'),
        findsOneWidget,
      );
      await tester.tap(find.text('OK'));
      await tester.pumpAndSettle();
      expect(find.textContaining('changed in its note'), findsNothing);
    });

    testWidgets('has a loading state', (tester) async {
      final library = FixtureLibraryRepository()..tasksGate = null;
      final blocs = await BrowseBlocs.start(library);
      final tasks = await blocs.tasksOf(read: false);
      await pumpRouted(tester, blocs.provide(const TasksPane(), tasks: tasks));
      expect(find.bySemanticsLabel('Loading tasks'), findsOneWidget);
    });

    testWidgets('says why there are none, by case', (tester) async {
      final library = FixtureLibraryRepository()
        ..tasksResult = const Right<ApplicationError, TaskBoard>(
          TaskBoard(groups: <TaskSection>[]),
        );
      await _show(tester, library: library);
      expect(find.text('No tasks yet'), findsOneWidget);
      expect(find.textContaining('Call Sam @2026-10-25'), findsOneWidget);
    });

    testWidgets('has an error state that tries again', (tester) async {
      final library = FixtureLibraryRepository()
        ..tasksResult = const Left<ApplicationError, TaskBoard>(
          ApplicationError.storage(message: 'Disk error.'),
        );
      await _show(tester, library: library);
      expect(find.text("The tasks didn't load"), findsOneWidget);
      library.tasksResult = const Right<ApplicationError, TaskBoard>(
        mockTaskBoard,
      );
      await tester.tap(find.text('Try again'));
      await tester.pumpAndSettle();
      expect(find.text('Send Sam the draft'), findsOneWidget);
    });

    testWidgets('filters by notebook and shows completed from its sheet', (
      tester,
    ) async {
      final blocs = await _show(tester);
      await tester.tap(find.bySemanticsLabel('Filter tasks'));
      await tester.pumpAndSettle();
      expect(find.text('All notebooks'), findsOneWidget);
      await tester.tap(find.text('Work').last);
      await tester.pumpAndSettle();
      expect(blocs.library.taskQueries.last.notebookId, MockIds.work);
      await tester.tap(find.text('Show Completed'));
      await tester.pumpAndSettle();
      expect(blocs.library.taskQueries.last.showDone, isTrue);
      await tester.tap(find.text('All notebooks'));
      await tester.pumpAndSettle();
      expect(blocs.library.taskQueries.last.notebookId, isNull);
    });

    testWidgets('names the notebook when it has no tasks', (tester) async {
      final library = FixtureLibraryRepository()
        ..tasksResult = const Right<ApplicationError, TaskBoard>(
          TaskBoard(groups: <TaskSection>[]),
        );
      final blocs = await _show(tester, library: library);
      await tester.tap(find.bySemanticsLabel('Filter tasks'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Work').last);
      await tester.pumpAndSettle();
      expect(find.text('No tasks in Work'), findsOneWidget);
      expect(blocs.library.taskQueries.last.notebookId, MockIds.work);
    });

    testWidgets('reads again when the library changes', (tester) async {
      final blocs = await _show(tester);
      final before = blocs.library.taskQueries.length;
      blocs.library.changeStream.add(<String>['n-a']);
      await tester.pump(const Duration(milliseconds: 300));
      expect(blocs.library.taskQueries.length, before + 1);
    });

    testWidgets('moves with the arrows, ticks with space, opens with return', (
      tester,
    ) async {
      final blocs = await BrowseBlocs.start();
      final tasks = await blocs.tasksOf();
      final router = await pumpRouted(
        tester,
        blocs.provide(const TasksPane(), tasks: tasks),
      );
      await tester.pumpAndSettle();
      await tester.sendKeyEvent(LogicalKeyboardKey.arrowDown);
      await tester.sendKeyEvent(LogicalKeyboardKey.arrowDown);
      await tester.pump();
      await tester.sendKeyEvent(LogicalKeyboardKey.space);
      await tester.pumpAndSettle();
      // Down twice: the second task, "Book the venue".
      expect(blocs.library.tasksSet.single.$1, 'n-b');
      await tester.sendKeyEvent(LogicalKeyboardKey.arrowUp);
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.pumpAndSettle();
      expect(
        whereIs(router),
        const BrowseLocation.tasks(noteId: 'n-a').location,
      );
    });

    testWidgets('the undo key takes the last tick back', (tester) async {
      final blocs = await _show(tester);
      await tester.sendKeyEvent(LogicalKeyboardKey.arrowDown);
      await tester.sendKeyEvent(LogicalKeyboardKey.space);
      await tester.pumpAndSettle();
      expect(blocs.undoNotice.state.notice, isNotNull);
      await tester.sendKeyDownEvent(LogicalKeyboardKey.controlLeft);
      await tester.sendKeyEvent(LogicalKeyboardKey.keyZ);
      await tester.sendKeyUpEvent(LogicalKeyboardKey.controlLeft);
      await tester.pumpAndSettle();
      expect(blocs.library.undone, <int>[7]);
    });

    testWidgets('is one node a screen reader can tick and open', (
      tester,
    ) async {
      final handle = tester.ensureSemantics();
      final blocs = await _show(tester);
      final node = tester.getSemantics(find.byType(TaskRow).first);
      final data = node.getSemanticsData();
      expect(data.label, contains('Send Sam the draft'));
      expect(data.label, contains('Due Oct 20'));
      expect(data.label, contains('Sync design · Projects'));
      expect(data.flagsCollection.isChecked.name, 'isFalse');
      expect(
        data.customSemanticsActionIds,
        isNotEmpty,
      );
      expect(find.byType(UndoNoticeBloc), findsNothing);
      expect(blocs.library.tasksSet, isEmpty);
      handle.dispose();
    });
  });
}
