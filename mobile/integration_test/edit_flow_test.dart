// Kanso — edit_flow_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:kanso/editor/views/widgets/completion_popup.dart';
import 'package:kanso/editor/views/widgets/new_property_row.dart';
import 'package:kanso/editor/views/widgets/properties_row.dart';
import 'package:kanso/search/views/widgets/highlighted_text.dart';

import 'helpers/editor_driver.dart';
import 'helpers/flow_driver.dart';
import 'helpers/seed_library.dart';

/// Editing: open a note, turn to Edit, type, wait past the autosave, leave and
/// come back; a list that continues, bold, undo and redo, a property, and the
/// completions of `[[` and `#`.
///
/// Run it with `flutter test integration_test/edit_flow_test.dart -d <device>`.
void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  Finder row(String title) => find.byWidgetPredicate(
    (widget) => widget is HighlightedText && widget.text == title,
  );

  testWidgets('a note is edited, saved by itself and kept', (tester) async {
    final semantics = tester.ensureSemantics();
    await startApp(tester);
    final seeded = await seedLibrary();
    await pumpUntil(tester, find.text('Weekly review'));

    // Open, edit, type, and wait past the autosave.
    await openForEditing(tester, seeded.sprintPlan);
    expect(find.byType(CompletionPopup), findsNothing);
    final controller = editorController(tester);
    expect(controller.text, contains('Ship the beta build to testers.'));
    controller.selection = TextSelection.collapsed(
      offset: controller.text.length,
    );
    await tester.pump();
    await typeText(tester, ' Then rest.');
    await pumpUntilStored(
      tester,
      seeded.sprintPlan,
      (body) => body.contains('Then rest.'),
    );

    // Leave and come back: the text is in the note.
    await tester.tap(find.bySemanticsLabel('Preview'));
    await pumpUntil(tester, find.textContaining('Then rest.'));
    await tester.tap(find.bySemanticsLabel('Edit'));
    await pumpUntil(tester, find.byType(PropertiesRow));
    expect(editorController(tester).text, contains('Then rest.'));

    // A list continues on Return.
    await typeText(tester, '\n\n- one');
    await typeText(tester, '\n');
    expect(editorController(tester).text, endsWith('- one\n- '));

    // Bold, from the bar.
    await typeText(tester, 'two');
    final text = editorController(tester).text;
    final start = text.lastIndexOf('two');
    editorController(tester).selection = TextSelection(
      baseOffset: start,
      extentOffset: start + 3,
    );
    await tester.pump();
    await tester.tap(accessoryKey('Bold'));
    await tester.pump();
    expect(editorController(tester).text, endsWith('- **two**'));

    // Undo takes the bold back; holding it redoes.
    await tester.tap(accessoryKey('Undo'));
    await tester.pump();
    expect(editorController(tester).text, endsWith('- two'));
    await tester.longPress(accessoryKey('Undo'));
    await tester.pump();
    expect(editorController(tester).text, endsWith('- **two**'));

    // A property.
    await tester.tap(find.byType(PropertiesRow));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Add property'));
    await tester.pumpAndSettle();
    final fields = find.descendant(
      of: find.byType(NewPropertyRow),
      matching: find.byType(EditableText),
    );
    await tester.enterText(fields.first, 'owner');
    await tester.enterText(fields.last, 'Ada');
    await tester.tap(find.bySemanticsLabel('Add'));
    await pumpUntil(tester, find.text('Properties · 1'));

    // `[[` completes with a title, and `#` with a tag.
    field(tester).focusNode.requestFocus();
    editorController(tester).selection = TextSelection.collapsed(
      offset: editorController(tester).text.length,
    );
    await tester.pump();
    await typeText(tester, '\n[[Weekly');
    await pumpUntil(tester, row('Weekly review'));
    await tester.tap(row('Weekly review'));
    await tester.pumpAndSettle();
    // The title is the link's text; where it leads is the core's to say.
    expect(
      editorController(tester).text,
      contains(RegExp(r'\[Weekly review\]\(\w+://note/[^)]+\)$')),
    );
    await typeText(tester, ' #to');
    await pumpUntil(tester, row('#todo'));
    await tester.tap(row('#todo'));
    await tester.pumpAndSettle();
    expect(editorController(tester).text, endsWith('#todo'));

    // All of it is saved, the property in the front matter.
    final stored = await pumpUntilStored(
      tester,
      seeded.sprintPlan,
      (body) => body.contains('#todo') && body.contains('owner'),
    );
    expect(stored, contains('[Weekly review]('));
    expect(stored, contains('- **two**'));
    expect(stored, startsWith('---\n'));
    expect(stored, contains('owner: Ada'));
    semantics.dispose();
  });
}
