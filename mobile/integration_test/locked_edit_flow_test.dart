// Kanso — locked_edit_flow_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:kanso/core/models/repos/abs/session_repository.dart';
import 'package:kanso/injection.dart';

import 'helpers/editor_driver.dart';
import 'helpers/flow_driver.dart';
import 'helpers/seed_library.dart';

/// A locked note is edited while it is open; when it locks the text is gone
/// from the screen, and what was typed is in the note, sealed.
///
/// Run it with `flutter test integration_test/locked_edit_flow_test.dart -d <device>`.
void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('edit an open locked note; lock it; the text is gone', (
    tester,
  ) async {
    final semantics = tester.ensureSemantics();
    await startApp(tester);
    final seeded = await seedLibrary();
    await pumpUntil(tester, find.text('Weekly review'));
    final session = getIt<SessionRepository>();

    // Closed, there is no editor for it.
    await openLink(
      tester,
      'kanso://open/note/${Uri.encodeComponent(seeded.privateJournal)}',
    );
    await pumpUntil(tester, find.text('This note is locked'));
    expect(find.bySemanticsLabel('Edit'), findsNothing);
    expect(await storedBody(seeded.privateJournal), isNull);

    // Open: it is edited like any other.
    final unlocked = await session.unlock(SeededLibrary.lockPassword);
    expect(unlocked.getOrElse(() => false), isTrue);
    // The screen reads the note again when it is opened.
    await openLink(
      tester,
      'kanso://open/note/${Uri.encodeComponent(seeded.weeklyReview)}',
    );
    await pumpUntil(tester, find.textContaining('Review the week'));
    await openForEditing(tester, seeded.privateJournal);
    expect(
      editorController(tester).text,
      contains(SeededLibrary.secretBody),
    );
    editorController(tester).selection = TextSelection.collapsed(
      offset: editorController(tester).text.length,
    );
    await typeText(tester, ' AND MORE');

    // It locks while the typing is not yet saved.
    final locked = await session.lock();
    expect(locked.isRight(), isTrue);
    await pumpUntil(tester, find.text('This note is locked'));
    expect(find.textContaining(SeededLibrary.secretBody), findsNothing);
    expect(find.textContaining('AND MORE'), findsNothing);
    expect(find.bySemanticsLabel('Edit'), findsNothing);
    expect(await storedBody(seeded.privateJournal), isNull);

    // Open again: what was typed was saved, sealed, before it closed.
    await session.unlock(SeededLibrary.lockPassword);
    final body = await pumpUntilStored(
      tester,
      seeded.privateJournal,
      (body) => body.contains('AND MORE'),
    );
    expect(body, contains(SeededLibrary.secretBody));
    semantics.dispose();
  });
}
