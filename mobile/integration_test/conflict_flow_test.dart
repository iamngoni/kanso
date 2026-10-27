// Kanso — conflict_flow_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:kanso/core/models/repos/abs/library_repository.dart';
import 'package:kanso/injection.dart';

import 'helpers/editor_driver.dart';
import 'helpers/flow_driver.dart';
import 'helpers/seed_library.dart';

/// A note changed elsewhere while it is edited: the question asked, and each
/// answer; and a change made while nothing was typed.
///
/// A note deleted elsewhere is not driven here: the library has no call that
/// deletes one for good (trashing is a move). The question for it is
/// covered by the core's tests, the BLoC's and the goldens.
///
/// Run it with `flutter test integration_test/conflict_flow_test.dart -d <device>`.
void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  /// Writes [body] over the note as another device's sync would, over the
  /// revision the library has now.
  Future<void> writeElsewhere(String id, String body) async {
    final library = getIt<LibraryRepository>();
    final current = (await library.note(id)).getOrElse(
      () => throw StateError('No note.'),
    );
    final saved = await library.save(id: id, baseRev: current.rev, body: body);
    saved.fold((error) => fail('The write failed: $error'), (_) {});
  }

  testWidgets('a note changed elsewhere while it is edited', (tester) async {
    final semantics = tester.ensureSemantics();
    await startApp(tester);
    final seeded = await seedLibrary();
    await pumpUntil(tester, find.text('Weekly review'));

    // Keep mine: typed text over theirs; theirs stays in the history.
    await openForEditing(tester, seeded.standupNotes);
    editorController(tester).selection = TextSelection.collapsed(
      offset: editorController(tester).text.length,
    );
    await typeText(tester, ' MINE');
    await writeElsewhere(seeded.standupNotes, '# Standup notes\n\nTHEIRS');
    await pumpUntil(
      tester,
      find.text(
        'This note was changed elsewhere while you were editing. The version '
        "you don't keep stays in its history.",
      ),
    );
    // Nothing is saved while the question is open.
    await tester.pump(const Duration(seconds: 2));
    expect(await storedBody(seeded.standupNotes), contains('THEIRS'));
    await tester.tap(find.text('Keep mine'));
    final kept = await pumpUntilStored(
      tester,
      seeded.standupNotes,
      (body) => body.contains('MINE'),
    );
    expect(kept, isNot(contains('THEIRS')));
    await pumpUntilGone(tester, find.text('Keep mine'));

    // Use theirs: the editor shows theirs, and mine is in the history.
    await typeText(tester, ' SECOND');
    await writeElsewhere(seeded.standupNotes, '# Standup notes\n\nTHEIRS TWO');
    await pumpUntil(tester, find.text('Use theirs'));
    await tester.tap(find.text('Use theirs'));
    await pumpUntilGone(tester, find.text('Use theirs'));
    expect(editorController(tester).text, contains('THEIRS TWO'));
    expect(editorController(tester).text, isNot(contains('SECOND')));
    expect(await storedBody(seeded.standupNotes), contains('THEIRS TWO'));

    // Nothing typed: what is elsewhere is just taken.
    await openForEditing(tester, seeded.gardenBeds);
    await writeElsewhere(seeded.gardenBeds, '# Garden beds\n\nFROM ELSEWHERE');
    await pumpUntilTrue(
      tester,
      () => editorController(tester).text.contains('FROM ELSEWHERE'),
      reason: 'the text from elsewhere in the editor',
    );
    expect(find.text('Keep mine'), findsNothing);
    semantics.dispose();
  });
}
