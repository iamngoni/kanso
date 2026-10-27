// Kanso — kanso_text_input_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/views/widgets/kanso_text_input.dart';

import '../../../helpers/pump.dart';

void main() {
  group('KansoTextInput', () {
    late TextEditingController controller;
    late FocusNode focus;

    setUp(() {
      controller = TextEditingController();
      focus = FocusNode();
      addTearDown(() {
        focus.dispose();
        controller.dispose();
      });
    });

    Future<List<String>> pumpInput(
      WidgetTester tester, {
      String? hint = 'Name',
    }) async {
      final submitted = <String>[];
      await pumpKanso(
        tester,
        SizedBox(
          width: 300,
          child: KansoTextInput(
            controller: controller,
            focusNode: focus,
            hint: hint,
            semanticLabel: 'The name',
            onSubmitted: submitted.add,
          ),
        ),
      );
      return submitted;
    }

    testWidgets('shows the hint until there is text', (tester) async {
      await pumpInput(tester);
      expect(find.text('Name'), findsOneWidget);
      await tester.enterText(find.byType(EditableText), 'owner');
      await tester.pump();
      expect(find.text('Name'), findsNothing);
      expect(controller.text, 'owner');
    });

    testWidgets('has no hint when it has none', (tester) async {
      await pumpInput(tester, hint: null);
      expect(find.byType(Text), findsNothing);
    });

    testWidgets('tells of the return key with the text', (tester) async {
      final submitted = await pumpInput(tester);
      await tester.enterText(find.byType(EditableText), 'owner');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      await tester.pump();
      expect(submitted, <String>['owner']);
    });

    testWidgets('is outlined in the accent while it has focus', (tester) async {
      await pumpInput(tester);
      BoxDecoration decoration() =>
          tester
                  .widget<Container>(
                    find
                        .descendant(
                          of: find.byType(KansoTextInput),
                          matching: find.byType(Container),
                        )
                        .first,
                  )
                  .decoration!
              as BoxDecoration;
      final before = (decoration().border! as Border).top;
      focus.requestFocus();
      await tester.pump();
      await tester.pump();
      final after = (decoration().border! as Border).top;
      expect(after.width, greaterThan(before.width));
      expect(after.color, isNot(before.color));
    });

    testWidgets('is a text field to a screen reader, with its label', (
      tester,
    ) async {
      final handle = tester.ensureSemantics();
      await pumpInput(tester);
      final node = tester.getSemantics(find.byType(KansoTextInput));
      expect(node.label, contains('The name'));
      expect(node.flagsCollection.isTextField, isTrue);
      handle.dispose();
    });
  });
}
