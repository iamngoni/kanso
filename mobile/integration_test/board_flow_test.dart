// Kanso — board_flow_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:kanso/board/views/widgets/board_pane.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/library_repository.dart';
import 'package:kanso/injection.dart';

import 'helpers/flow_driver.dart';
import 'helpers/seed_library.dart';

/// Boards as a person uses them: the notes by status with a card moved to
/// another column, and the tasks board with a task ticked off.
///
/// Run it with `flutter test integration_test/board_flow_test.dart -d <device>`.
void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  Finder inBoard(String label) => find.descendant(
    of: find.byType(BoardPane),
    matching: find.text(label),
  );

  double left(WidgetTester tester, Finder finder) =>
      tester.getTopLeft(finder.first).dx;

  /// Moves the card with [title] to [lane] by the move sheet: a touch and
  /// hold on a phone, the card's button where cards are dragged.
  Future<void> moveTo(WidgetTester tester, String title, String lane) async {
    final more = find.descendant(
      of: find.ancestor(
        of: inBoard(title),
        matching: find.byType(GestureDetector),
      ),
      matching: find.bySemanticsLabel('Move to…'),
    );
    if (more.evaluate().isNotEmpty) {
      await tester.tap(more.first);
    } else {
      await tester.longPress(inBoard(title));
    }
    await pumpUntil(tester, find.text('Move to'));
    await tester.tap(find.text(lane).last);
    await tester.pumpAndSettle();
  }

  testWidgets('move a note to a status and tick off a task', (tester) async {
    final semantics = tester.ensureSemantics();
    await startApp(tester);
    await seedLibrary();
    final library = getIt<LibraryRepository>();
    final Either<ApplicationError, Saved> chores = await library.create(
      const NewNote(
        title: 'Chores',
        body: '# Chores\n\n- [ ] Water the beans\n- [ ] Order seeds\n',
      ),
    );
    expect(chores.isRight(), isTrue);
    await pumpUntil(tester, inList('Chores'));

    // The board of All Notes, from the list's more menu.
    await tester.tap(find.bySemanticsLabel('More'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Notes').last);
    await pumpUntil(tester, find.byType(BoardPane));
    await pumpUntil(tester, inBoard('Sprint plan'));
    expect(inBoard('No status'), findsOneWidget);
    expect(inBoard('Active'), findsOneWidget);

    // Sprint plan has no status; move it to Active.
    final before = left(tester, inBoard('Sprint plan'));
    await moveTo(tester, 'Sprint plan', 'Active');
    await pumpUntilTrue(
      tester,
      () => left(tester, inBoard('Sprint plan')) > before,
      reason: 'the card in the Active column',
    );

    // The tasks board: two tasks to do; one is ticked off.
    await tester.tap(inBoard('Tasks'));
    await pumpUntil(tester, inBoard('Water the beans'));
    expect(inBoard('To do'), findsOneWidget);
    expect(inBoard('Done'), findsOneWidget);
    final todo = left(tester, inBoard('Water the beans'));
    await tester.tap(find.bySemanticsLabel('Mark done').first);
    await pumpUntilTrue(
      tester,
      () =>
          inBoard('Water the beans').evaluate().isNotEmpty &&
          left(tester, inBoard('Water the beans')) > todo,
      reason: 'the task in the Done column',
    );
    expect(find.bySemanticsLabel('Mark not done'), findsWidgets);
    expect(find.byType(Scaffold), findsWidgets);

    semantics.dispose();
  });
}
