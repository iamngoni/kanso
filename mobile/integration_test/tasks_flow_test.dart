// Kanso — tasks_flow_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/library_repository.dart';
import 'package:kanso/injection.dart';
import 'package:kanso/note/views/pages/note_page.dart';
import 'package:kanso/tasks/views/widgets/task_row.dart';
import 'package:kanso/tasks/views/widgets/tasks_pane.dart';

import 'helpers/flow_driver.dart';
import 'helpers/seed_library.dart';

/// The Tasks place as a person uses it, on a library of the app's own: open
/// it from the library, tick a task and see it struck, take that back with
/// Undo, and open a task's note.
///
/// Run it with `flutter test integration_test/tasks_flow_test.dart -d <device>`.
void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  Finder inTasks(String label) =>
      find.descendant(of: find.byType(TasksPane), matching: find.text(label));

  testWidgets('open Tasks, tick a task, Undo, open its note', (tester) async {
    final semantics = tester.ensureSemantics();
    await startApp(tester);
    await seedLibrary();
    final library = getIt<LibraryRepository>();
    final saved =
        (await library.create(
          const NewNote(
            title: 'Errands',
            body: '# Errands\n\n- [ ] Buy stamps\n- [ ] Post the parcel\n',
          ),
        )).fold<Saved>(
          (ApplicationError error) =>
              throw StateError('Seeding failed: $error'),
          (value) => value,
        );

    await openLibrary(tester);
    await tester.tap(inLibrary('Tasks'));
    await tester.pumpAndSettle();
    await pumpUntil(tester, inTasks('Buy stamps'));
    expect(inTasks('Post the parcel'), findsOneWidget);
    expect(inTasks('No date'), findsOneWidget);

    TextDecoration? decoration(String text) =>
        tester.widget<Text>(inTasks(text)).style?.decoration;
    expect(decoration('Buy stamps'), isNull);

    // Tick it: it stays where it is, struck through, and the notice offers
    // Undo.
    final row = tester.getTopLeft(
      find.ancestor(of: inTasks('Buy stamps'), matching: find.byType(TaskRow)),
    );
    await tester.tapAt(row + const Offset(30, 24));
    await pumpUntil(tester, find.text('Ticked off “Buy stamps”'));
    await pumpUntilTrue(
      tester,
      () => decoration('Buy stamps') == TextDecoration.lineThrough,
      reason: 'the task to be struck through',
    );
    expect(inTasks('Buy stamps'), findsOneWidget);

    // Undo: it is open again.
    await tester.tap(find.text('Undo'));
    await pumpUntilTrue(
      tester,
      () => decoration('Buy stamps') == null,
      reason: 'the task to be open again',
    );

    // Open its note.
    await tester.tap(inTasks('Post the parcel'));
    await pumpUntil(tester, find.byType(NotePage));
    expect(find.text('Errands'), findsWidgets);
    expect(saved.id, isNotEmpty);
    semantics.dispose();
  });
}
