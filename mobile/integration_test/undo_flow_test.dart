// Kanso — undo_flow_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';

import 'helpers/flow_driver.dart';
import 'helpers/seed_library.dart';

/// Taking a change back as a person does, on a library of the app's own: a
/// note swiped to the trash comes back with Undo, and a notice that is not
/// answered goes by itself.
///
/// Run it with `flutter test integration_test/undo_flow_test.dart -d <device>`.
void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('swipe a note to the trash, then Undo brings it back', (
    tester,
  ) async {
    final semantics = tester.ensureSemantics();
    await startApp(tester);
    await seedLibrary();
    await pumpUntil(tester, inList('Garden beds'));

    // Trash it: the notice says so, with Undo.
    await tester.drag(inList('Garden beds'), const Offset(-300, 0));
    await pumpUntilGone(tester, inList('Garden beds'));
    await pumpUntil(tester, find.text('Moved 1 note to the trash.'));
    expect(find.text('Undo'), findsOneWidget);

    // Undo: the note is back in the list, and the notice is gone.
    await tester.tap(find.text('Undo'));
    await pumpUntil(tester, inList('Garden beds'));
    await pumpUntilGone(tester, find.text('Moved 1 note to the trash.'));

    // Trash it again and leave the notice alone: it goes by itself.
    await tester.drag(inList('Garden beds'), const Offset(-300, 0));
    await pumpUntilGone(tester, inList('Garden beds'));
    await pumpUntil(tester, find.text('Moved 1 note to the trash.'));
    await pumpUntilTrue(
      tester,
      () => find.text('Moved 1 note to the trash.').evaluate().isEmpty,
      reason: 'the notice to go by itself',
    );
    expect(inList('Garden beds'), findsNothing);
    semantics.dispose();
  });
}
