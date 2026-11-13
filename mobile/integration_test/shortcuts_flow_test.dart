// Kanso — shortcuts_flow_test.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:kanso/capture/views/widgets/capture_task_form.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/repos/abs/library_repository.dart';
import 'package:kanso/core/router/kanso_router.dart';
import 'package:kanso/injection.dart';
import 'package:kanso/search/views/pages/search_page.dart';

import 'helpers/flow_driver.dart';
import 'helpers/seed_library.dart';

/// The Android launcher shortcuts and the Add task Quick Settings tile open
/// these links (#390 slice 19): `kanso://open/new?task=1` shows the capture
/// sheet in Task mode over where the app is, and a task typed there is
/// written; `kanso://open/search` opens Search.
///
/// Run it with `flutter test integration_test/shortcuts_flow_test.dart -d
/// <device>`.
void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  String where() => getIt<KansoRouter>()
      .config
      .routerDelegate
      .currentConfiguration
      .uri
      .toString();

  testWidgets('Add task opens the Task sheet over the app and writes a task', (
    tester,
  ) async {
    final semantics = tester.ensureSemantics();
    await startApp(tester);
    final seeded = await seedLibrary();
    await pumpUntil(tester, inList('Weekly review'));

    // Over a notebook's list: the sheet opens in Task mode and the list
    // stays where it was.
    await openLink(
      tester,
      'kanso://open/notebook/${Uri.encodeComponent(seeded.garden)}',
    );
    await pumpUntil(tester, inList('Garden beds'));
    final before = where();
    await openLink(tester, 'kanso://open/new?task=1');
    await pumpUntil(tester, find.byType(CaptureTaskForm));
    expect(where(), before);
    expect(inList('Garden beds'), findsWidgets);

    // A task typed there is written to today's daily note.
    const task = 'water the shortcut seedlings';
    await tester.enterText(
      find.descendant(
        of: find.byType(CaptureTaskForm),
        matching: find.byType(EditableText),
      ),
      task,
    );
    await tester.pump();
    await tester.tap(find.text('Add task'));
    await pumpUntil(tester, find.textContaining('Added to'));
    final library = getIt<LibraryRepository>();
    final deadline = DateTime.now().add(const Duration(seconds: 15));
    var found = false;
    while (!found && DateTime.now().isBefore(deadline)) {
      final hits = (await library.list(
        const ListQuery(
          place: ListPlace.all(),
          pinnedOnly: false,
          sort: SortKey.updated,
          descending: true,
        ),
      )).getOrElse(() => throw StateError('The library did not list.'));
      for (final row in hits.rows.take(5)) {
        final note = (await library.note(row.id)).fold((_) => null, (n) => n);
        if ((note?.body ?? '').contains('- [ ] $task')) {
          found = true;
          break;
        }
      }
      if (!found) {
        await tester.pump(const Duration(milliseconds: 200));
      }
    }
    expect(found, isTrue, reason: 'The task was never written.');

    // Closed, the app is where it was.
    tester.state<NavigatorState>(find.byType(Navigator).first).pop();
    await tester.pumpAndSettle();
    expect(find.byType(CaptureTaskForm), findsNothing);
    expect(where(), before);

    // Search opens Search.
    await openLink(tester, 'kanso://open/search');
    await pumpUntil(tester, find.byType(SearchPage));
    expect(where(), startsWith('/search'));
    semantics.dispose();
  });
}
