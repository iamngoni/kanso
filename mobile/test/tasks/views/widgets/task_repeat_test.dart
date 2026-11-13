// Kanso — task_repeat_test.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/tasks/views/widgets/task_repeat_chip.dart';
import 'package:kanso/tasks/views/widgets/task_row.dart';
import 'package:kanso/tasks/views/widgets/tasks_pane.dart';

import '../../../helpers/browse_blocs.dart';
import '../../../helpers/fixture_library_repository.dart';
import '../../../helpers/pump.dart';
import '../../../helpers/pump_routed.dart';

const String _long =
    'Film the Pixel review: the unboxing, the camera tests in the studio and '
    'the low-light walk, then cut the b-roll for the sponsor segment and '
    'send the rough cut to Sam for notes before Friday';

TaskItem _task({
  String text = 'Film Pixel review',
  String? repeat = 'every week',
  bool checked = false,
}) => TaskItem(
  key: 'film',
  noteId: 'n-film',
  noteTitle: 'Studio week',
  notebookId: 'b-studio',
  notebookName: 'Studio',
  line: 3,
  text: text,
  sourceText: '$text @2026-10-23T15:00 ${repeat ?? ''}'.trim(),
  checked: checked,
  dueLabel: 'Fri 15:00',
  dueTone: DueTone.plain,
  repeatLabel: repeat,
);

TaskBoard _board(TaskItem task) => TaskBoard(
  groups: <TaskSection>[
    TaskSection(
      group: TaskGroupKind.upcoming,
      label: 'Upcoming',
      items: <TaskItem>[task],
    ),
  ],
);

void main() {
  group('TaskRow repeat chip', () {
    testWidgets('a repeating task shows how it repeats beside its date', (
      tester,
    ) async {
      await pumpKanso(
        tester,
        TaskRow(task: _task(), onTick: () {}, onOpen: () {}),
      );
      expect(find.byType(TaskRepeatChip), findsOneWidget);
      expect(find.text('every week'), findsOneWidget);
      expect(find.text('Fri 15:00'), findsOneWidget);
    });

    testWidgets('a task that does not repeat has no chip', (tester) async {
      await pumpKanso(
        tester,
        TaskRow(task: _task(repeat: null), onTick: () {}, onOpen: () {}),
      );
      expect(find.byType(TaskRepeatChip), findsNothing);
    });

    testWidgets('a screen reader hears that it repeats', (tester) async {
      final handle = tester.ensureSemantics();
      await pumpKanso(
        tester,
        TaskRow(
          task: _task(),
          onTick: () {},
          onOpen: () {},
          onLongPress: () {},
        ),
      );
      final label = tester
          .getSemantics(find.byType(TaskRow))
          .getSemanticsData()
          .label;
      expect(label, contains('Repeats every week'));
      handle.dispose();
    });

    testWidgets('holding the row asks for its whole text', (tester) async {
      var held = 0;
      await pumpKanso(
        tester,
        TaskRow(
          task: _task(),
          onTick: () {},
          onOpen: () {},
          onLongPress: () => held++,
        ),
      );
      await tester.longPress(find.text('Film Pixel review'));
      expect(held, 1);
    });
  });

  group('Tasks place', () {
    Future<BrowseBlocs> show(
      WidgetTester tester,
      TaskItem task, {
      FixtureLibraryRepository? library,
    }) async {
      final fixture = (library ?? FixtureLibraryRepository())
        ..tasksResult = Right<ApplicationError, TaskBoard>(_board(task));
      final blocs = await BrowseBlocs.start(fixture);
      final tasks = await blocs.tasksOf();
      await pumpRouted(tester, blocs.provide(const TasksPane(), tasks: tasks));
      await tester.pumpAndSettle();
      return blocs;
    }

    testWidgets('a long press shows the full text and how it repeats', (
      tester,
    ) async {
      await show(tester, _task(text: _long));
      // The row cuts the text at three lines; the sheet has all of it.
      expect(find.text(_long), findsOneWidget);
      await tester.longPress(find.text(_long));
      await tester.pumpAndSettle();
      expect(find.text(_long), findsNWidgets(2));
      expect(find.text('Repeats every week'), findsOneWidget);
      expect(find.text('Open note'), findsOneWidget);
      expect(find.text('Studio week · Studio'), findsNWidgets(2));
    });

    testWidgets('the sheet closes with Done', (tester) async {
      await show(tester, _task(text: _long));
      await tester.longPress(find.text(_long));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Done'));
      await tester.pumpAndSettle();
      expect(find.text('Repeats every week'), findsNothing);
      expect(find.text(_long), findsOneWidget);
    });

    testWidgets('a task that does not repeat says nothing of repeating', (
      tester,
    ) async {
      await show(tester, _task(repeat: null));
      await tester.longPress(find.text('Film Pixel review'));
      await tester.pumpAndSettle();
      expect(find.textContaining('Repeats'), findsNothing);
      expect(find.text('Open note'), findsOneWidget);
    });

    testWidgets('ticking a repeating task writes the task as the library '
        'knows it, once; the library writes the next one', (tester) async {
      final blocs = await show(tester, _task());
      final row = tester.getTopLeft(find.byType(TaskRow).first);
      await tester.tapAt(row + const Offset(30, 30));
      await tester.pumpAndSettle();
      // The tick is one write of the line with its repeat words, so the
      // library can write the next occurrence (`tick_task_at`).
      expect(blocs.library.tasksSet, <(String, int, String, bool)>[
        ('n-film', 3, 'Film Pixel review @2026-10-23T15:00 every week', true),
      ]);
    });
  });
}
