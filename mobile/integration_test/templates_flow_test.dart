// Kanso — templates_flow_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:kanso/note/views/pages/note_page.dart';
import 'package:kanso/templates/views/widgets/template_picker_sheet.dart';

import 'helpers/flow_driver.dart';
import 'helpers/seed_library.dart';

/// Templates as a person uses them: a template made, a note made from it,
/// and the daily template set and cleared in Settings.
///
/// Run it with `flutter test integration_test/templates_flow_test.dart -d <device>`.
void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  String openNote(WidgetTester tester) =>
      tester.widget<NotePage>(find.byType(NotePage)).noteId;

  testWidgets('make a template, make a note from it, set the daily one', (
    tester,
  ) async {
    final semantics = tester.ensureSemantics();
    await startApp(tester);
    await seedLibrary();
    await pumpUntil(tester, inList('Weekly review'));

    // No template yet.
    await tester.tap(find.bySemanticsLabel('New').first);
    await tester.pumpAndSettle();
    await tester.tap(find.text('New from template…'));
    await pumpUntil(tester, find.text('No templates yet'));
    await tester.tapAt(const Offset(5, 5));
    await pumpUntilGone(tester, find.byType(TemplatePickerSheet));

    // A new template opens as a note.
    await tester.tap(find.bySemanticsLabel('New').first);
    await tester.pumpAndSettle();
    await tester.tap(find.text('New template'));
    await pumpUntil(tester, find.byType(NotePage));
    final template = openNote(tester);
    await leaveNote(tester);

    // A note from it: another note, opened.
    await tester.tap(find.bySemanticsLabel('New').first);
    await tester.pumpAndSettle();
    await tester.tap(find.text('New from template…'));
    await pumpUntil(tester, find.byType(TemplatePickerSheet));
    await pumpUntil(tester, find.text('Untitled'));
    await tester.tap(find.text('Untitled'));
    await pumpUntilTrue(
      tester,
      () =>
          find.byType(NotePage).evaluate().isNotEmpty &&
          openNote(tester) != template,
      reason: 'the note made from the template',
    );
    await leaveNote(tester);

    // The daily template: None, then the template, then None again.
    await openLink(tester, '/settings');
    await pumpUntil(tester, find.text('Daily template'));
    expect(find.text('None'), findsOneWidget);
    await tester.tap(find.text('Daily template'));
    await pumpUntil(tester, find.byType(TemplatePickerSheet));
    await tester.tap(find.text('Untitled'));
    await pumpUntilGone(tester, find.byType(TemplatePickerSheet));
    await pumpUntil(tester, find.text('Untitled'));
    expect(find.text('None'), findsNothing);
    await tester.tap(find.text('Daily template'));
    await pumpUntil(tester, find.byType(TemplatePickerSheet));
    await tester.tap(find.text('None'));
    await pumpUntilGone(tester, find.byType(TemplatePickerSheet));
    await pumpUntil(tester, find.text('None'));

    semantics.dispose();
  });
}
