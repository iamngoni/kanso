// Kanso — editor_driver.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/repos/abs/library_repository.dart';
import 'package:kanso/editor/views/controllers/kanso_markdown_controller.dart';
import 'package:kanso/editor/views/widgets/accessory_key.dart';
import 'package:kanso/editor/views/widgets/kanso_editable_text.dart';
import 'package:kanso/injection.dart';

import 'flow_driver.dart';

/// Opens the note [id] from outside the app and turns it to Edit.
Future<void> openForEditing(WidgetTester tester, String id) async {
  await openLink(tester, 'kanso://open/note/${Uri.encodeComponent(id)}');
  await pumpUntil(tester, find.bySemanticsLabel('Edit'));
  await tester.tap(find.bySemanticsLabel('Edit'));
  await pumpUntil(tester, find.byType(KansoEditableText));
  // The keyboard is up once the field has the focus.
  await pumpUntilTrue(
    tester,
    () => field(tester).focusNode.hasFocus,
    reason: 'the editor to have the focus',
  );
}

/// The field of the note's text.
EditableText field(WidgetTester tester) =>
    tester.widget<EditableText>(find.byType(KansoEditableText));

/// The controller of that field.
KansoMarkdownController editorController(WidgetTester tester) =>
    field(tester).controller as KansoMarkdownController;

/// Types [text] at the caret as the keyboard does.
Future<void> typeText(WidgetTester tester, String text) async {
  final controller = editorController(tester);
  final value = controller.value;
  final at = value.selection.isValid
      ? value.selection.start
      : value.text.length;
  final end = value.selection.isValid ? value.selection.end : at;
  final next = value.text.replaceRange(at, end, text);
  tester.testTextInput.updateEditingValue(
    TextEditingValue(
      text: next,
      selection: TextSelection.collapsed(offset: at + text.length),
    ),
  );
  await tester.pump();
}

/// The key of the keyboard bar that says [label].
Finder accessoryKey(String label) => find.byWidgetPredicate(
  (widget) => widget is AccessoryKey && widget.semanticLabel == label,
);

/// The body of the note as the library has it, or null.
Future<String?> storedBody(String id) async {
  final result = await getIt<LibraryRepository>().note(id);
  return result.fold((_) => null, (note) => note.body);
}

/// Waits for the library to hold a body of the note [id] that [test] likes,
/// for up to [timeout], pumping as it waits.
Future<String> pumpUntilStored(
  WidgetTester tester,
  String id,
  bool Function(String body) test, {
  Duration timeout = const Duration(seconds: 15),
}) async {
  final deadline = DateTime.now().add(timeout);
  while (DateTime.now().isBefore(deadline)) {
    final body = await storedBody(id);
    if (body != null && test(body)) {
      return body;
    }
    await tester.pump(const Duration(milliseconds: 100));
  }
  fail('The library never held what was typed in $id: ${await storedBody(id)}');
}
