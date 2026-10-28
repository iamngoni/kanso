// Kanso — note_window_flow_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:kanso/editor/views/widgets/editor_title_field.dart';
import 'package:kanso/injection.dart';
// Imported so that the entry point of a note's window is in the program the
// native side starts its second engine from.
import 'package:kanso/note_window_main.dart' show noteWindowMain;
import 'package:kanso/window/models/data/note_window_info.dart';
import 'package:kanso/window/models/repos/abs/note_window_repository.dart';

import 'helpers/editor_driver.dart';
import 'helpers/flow_driver.dart';
import 'helpers/seed_library.dart';

/// A note in a window of its own: opened from the main window by the
/// repository (an iPadOS scene, an Android task), it runs a second Flutter
/// engine on the same library; the title typed in the main window's editor
/// reaches it through the library, the window of a locked note is named
/// "Locked note", opening a note that has a window brings that window
/// forward and makes no second one, and the window closes.
///
/// What the test can see of the other engine is what the native side knows
/// of its window (`openWindows`: the note, and the name the Dart side of the
/// window gave it once it had read the note). Typing in the window itself
/// cannot be driven from here; the sharing of what is typed is checked in
/// the Rust core (`crates/kanso-mobile/tests/windows.rs`).
///
/// Run it with
/// `flutter test integration_test/note_window_flow_test.dart -d <device>`.
/// On a device that shows one window (an iPhone) there is nothing to do.
void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();
  // Keeps the entry point from being dropped.
  const entry = noteWindowMain;

  Future<List<NoteWindowInfo>> windowsOf(NoteWindowRepository windows) async {
    return (await windows.openWindows()).fold(
      (error) => throw StateError('openWindows failed: $error'),
      (open) => open,
    );
  }

  Future<List<NoteWindowInfo>> pumpUntilWindows(
    WidgetTester tester,
    NoteWindowRepository windows,
    bool Function(List<NoteWindowInfo> open) test, {
    Duration timeout = const Duration(seconds: 30),
  }) async {
    final deadline = DateTime.now().add(timeout);
    var open = await windowsOf(windows);
    while (!test(open) && DateTime.now().isBefore(deadline)) {
      await tester.pump(const Duration(milliseconds: 200));
      open = await windowsOf(windows);
    }
    expect(test(open), isTrue, reason: 'the windows were $open');
    return open;
  }

  testWidgets('a note opens in a window of its own, follows the main '
      'window, and closes', (tester) async {
    expect(entry, isNotNull);
    final semantics = tester.ensureSemantics();
    await startApp(tester);
    final seeded = await seedLibrary();
    await pumpUntil(tester, find.text('Weekly review'));
    final windows = getIt<NoteWindowRepository>();
    if (!windows.supported) {
      // ignore: avoid_print
      print('This device shows one window: nothing to open.');
      semantics.dispose();
      return;
    }

    // The note, edited in the main window.
    await openForEditing(tester, seeded.sprintPlan);

    // Open it in a window of its own: a second engine reads the note from
    // the same library and names its window after it.
    expect((await windows.open(seeded.sprintPlan)).isRight(), isTrue);
    await pumpUntilWindows(
      tester,
      windows,
      (open) =>
          open.length == 1 &&
          open.single.noteId == seeded.sprintPlan &&
          open.single.title == 'Sprint plan',
    );

    // Opening a note that has a window brings it forward, and makes no
    // second.
    expect((await windows.open(seeded.sprintPlan)).isRight(), isTrue);
    await tester.pump(const Duration(seconds: 2));
    expect(await windowsOf(windows), hasLength(1));

    // A title typed in the main window is saved by it, and the other
    // engine's window is named after it.
    final title = find.descendant(
      of: find.byType(EditorTitleField),
      matching: find.byType(EditableText),
    );
    await tester.enterText(title, 'Sprint plan, shared');
    await tester.pump();
    await pumpUntilStored(
      tester,
      seeded.sprintPlan,
      (body) => body.contains('Ship the beta build'),
    );
    await pumpUntilWindows(
      tester,
      windows,
      (open) =>
          open.length == 1 && open.single.title == 'Sprint plan, shared',
    );

    // The window of a locked note is named "Locked note", whatever its
    // title.
    expect((await windows.open(seeded.privateJournal)).isRight(), isTrue);
    final both = await pumpUntilWindows(
      tester,
      windows,
      (open) =>
          open.length == 2 &&
          open.any(
            (window) =>
                window.noteId == seeded.privateJournal &&
                window.title == 'Locked note',
          ),
    );
    expect(
      both.any((window) => window.title.contains('Private')),
      isFalse,
      reason: "a locked note's title is kept out of its window name",
    );

    // Close them.
    expect(
      (await windows.closeWindowOf(seeded.privateJournal)).isRight(),
      isTrue,
    );
    expect(
      (await windows.closeWindowOf(seeded.sprintPlan)).isRight(),
      isTrue,
    );
    await pumpUntilWindows(tester, windows, (open) => open.isEmpty);
    semantics.dispose();
  });
}
