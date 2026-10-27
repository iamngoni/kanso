// Kanso — new_note_flow_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/repos/abs/library_repository.dart';
import 'package:kanso/core/views/widgets/kanso_icon_button.dart';
import 'package:kanso/editor/views/widgets/editor_title_field.dart';
import 'package:kanso/injection.dart';
import 'package:kanso/note_list/views/widgets/new_note_button.dart';
import 'package:kanso/settings/views/widgets/default_notebook_pane.dart';
import 'package:kanso/settings/views/widgets/settings_pane.dart';

import 'helpers/editor_driver.dart';
import 'helpers/flow_driver.dart';
import 'helpers/seed_library.dart';

/// New notes and notebooks: the Default notebook chosen in Settings, a note
/// made with + that opens ready to write its title, a note made from a tag's
/// list, ⌘N, and a new notebook.
///
/// Run it with `flutter test integration_test/new_note_flow_test.dart -d
/// <device>`.
void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  Finder iconButton(String label) => find.byWidgetPredicate(
    (widget) => widget is KansoIconButton && widget.semanticLabel == label,
  );

  /// The button that makes a note: afloat over a phone's list, in the bar
  /// of a list beside its note.
  Finder newNoteButton() => find.byType(NewNoteButton).evaluate().isNotEmpty
      ? find.byType(NewNoteButton)
      : iconButton('New note');

  /// The note in the library titled [title], once it is there.
  Future<NoteRow> noteTitled(WidgetTester tester, String title) async {
    final library = getIt<LibraryRepository>();
    final deadline = DateTime.now().add(const Duration(seconds: 15));
    while (DateTime.now().isBefore(deadline)) {
      final listed = await library.list(
        const ListQuery(
          place: ListPlace.all(),
          pinnedOnly: false,
          sort: SortKey.updated,
          descending: true,
        ),
      );
      final found = listed
          .getOrElse(() => throw StateError('The library did not list.'))
          .rows
          .where((row) => row.title == title)
          .firstOrNull;
      if (found != null) {
        return found;
      }
      await tester.pump(const Duration(milliseconds: 100));
    }
    fail('The library never held a note titled $title.');
  }

  Future<NoteView> read(String id) async =>
      (await getIt<LibraryRepository>().note(
        id,
      )).getOrElse(() => throw StateError('The note could not be read.'));

  /// Whether the title field of the editor has the focus.
  bool titleFocused(WidgetTester tester) {
    final inTitle = find.descendant(
      of: find.byType(EditorTitleField),
      matching: find.byType(EditableText),
    );
    return tester.widget<EditableText>(inTitle).focusNode.hasFocus;
  }

  /// Writes [title] in the title of the note being edited and then [text] in
  /// its text, and waits until the library holds both.
  Future<String> write(WidgetTester tester, String title, String text) async {
    await pumpUntil(tester, find.byType(EditorTitleField));
    // The title has the focus the moment the editor is there.
    await tester.pumpAndSettle();
    expect(titleFocused(tester), isTrue);
    await tester.enterText(
      find.descendant(
        of: find.byType(EditorTitleField),
        matching: find.byType(EditableText),
      ),
      title,
    );
    await tester.pump();
    field(tester).focusNode.requestFocus();
    await tester.pump();
    await typeText(tester, text);
    final made = await noteTitled(tester, title);
    await pumpUntilStored(tester, made.id, (body) => body.contains(text));
    return made.id;
  }

  testWidgets('notes and notebooks are made, and go where they should', (
    tester,
  ) async {
    final semantics = tester.ensureSemantics();
    await startApp(tester);
    final seeded = await seedLibrary();
    await pumpUntil(tester, find.text('Weekly review'));

    // Settings › General › Default notebook: Garden.
    await openLibrary(tester);
    await tester.tap(find.bySemanticsLabel('Settings'));
    await tester.pumpAndSettle();
    expect(find.text('First notebook'), findsOneWidget);
    await tester.tap(find.text('Default notebook'));
    await tester.pumpAndSettle();
    await tester.tap(
      find.descendant(
        of: find.byType(DefaultNotebookPane),
        matching: find.text('Garden'),
      ),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.bySemanticsLabel('Back').last);
    await tester.pumpAndSettle();
    expect(
      find.descendant(
        of: find.byType(SettingsPane),
        matching: find.text('Garden'),
      ),
      findsOneWidget,
    );
    await tester.tap(find.bySemanticsLabel('Close'));
    await tester.pumpAndSettle();

    // + makes a note in the default notebook, open to write its title.
    await tester.tap(newNoteButton());
    await tester.pumpAndSettle();
    final first = await write(tester, 'Planting list', 'Carrots and leeks.');
    expect((await read(first)).notebookId, seeded.garden);
    await tester.tap(find.bySemanticsLabel('Preview'));
    await pumpUntil(tester, find.textContaining('Carrots and leeks.'));
    await leaveNote(tester);
    await pumpUntil(tester, inList('Planting list'));

    // A tag's list tags the new note, and the notebook is the default one.
    await openLibrary(tester);
    await tester.tap(inLibrary('todo'));
    await tester.pumpAndSettle();
    await tester.tap(newNoteButton());
    await tester.pumpAndSettle();
    final tagged = await write(tester, 'Tagged here', 'Made from a tag.');
    final tags = (await getIt<LibraryRepository>().tags()).getOrElse(
      () => const <TagRow>[],
    );
    final todo = tags.firstWhere((tag) => tag.name == 'todo');
    expect((await read(tagged)).tags, contains(todo.id));
    await tester.tap(find.bySemanticsLabel('Preview'));
    await pumpUntil(tester, find.textContaining('Made from a tag.'));
    await leaveNote(tester);

    // The shortcut makes one as well.
    final secondary = defaultTargetPlatform == TargetPlatform.iOS
        ? LogicalKeyboardKey.metaLeft
        : LogicalKeyboardKey.controlLeft;
    await tester.sendKeyDownEvent(secondary);
    await tester.sendKeyEvent(LogicalKeyboardKey.keyN);
    await tester.sendKeyUpEvent(secondary);
    await tester.pumpAndSettle();
    final keyed = await write(tester, 'By shortcut', 'Typed with a key.');
    expect((await read(keyed)).tags, contains(todo.id));
    await tester.tap(find.bySemanticsLabel('Preview'));
    await pumpUntil(tester, find.textContaining('Typed with a key.'));
    await leaveNote(tester);

    // A new notebook, from the library, and its list opens.
    await openLibrary(tester);
    await tester.tap(iconButton('New notebook'));
    await tester.pumpAndSettle();
    await tester.enterText(find.byType(EditableText).last, 'Recipes');
    await tester.pump();
    await tester.tap(find.text('Create'));
    await pumpUntil(tester, find.text('Recipes'));
    final notebooks = (await getIt<LibraryRepository>().notebooks()).getOrElse(
      () => const <NotebookRow>[],
    );
    expect(notebooks.where((book) => book.name == 'Recipes'), hasLength(1));
    expect(find.text('No notes in Recipes yet'), findsOneWidget);

    // A note made in an empty notebook's own list goes in it, whatever the
    // default is.
    await tester.tap(newNoteButton());
    await tester.pumpAndSettle();
    final recipe = await write(tester, 'Soup', 'Stock first.');
    final recipes = notebooks.firstWhere((book) => book.name == 'Recipes');
    expect((await read(recipe)).notebookId, recipes.id);
    semantics.dispose();
  });
}
