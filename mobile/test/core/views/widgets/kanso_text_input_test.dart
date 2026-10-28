// Kanso — kanso_text_input_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/views/widgets/kanso_text_input.dart';

import '../../../helpers/pump.dart';

Future<TextEditingController> _pump(
  WidgetTester tester, {
  String text = '',
  bool enabled = true,
  List<String>? changes,
  List<String>? submitted,
}) async {
  final controller = TextEditingController(text: text);
  addTearDown(controller.dispose);
  await pumpKanso(
    tester,
    SizedBox(
      width: 320,
      child: KansoTextInput(
        controller: controller,
        hint: 'Username',
        enabled: enabled,
        onChanged: changes?.add,
        onSubmitted: submitted?.add,
      ),
    ),
  );
  return controller;
}

void main() {
  group('KansoTextInput', () {
    testWidgets('shows the hint until there is text', (tester) async {
      final controller = await _pump(tester);
      expect(find.text('Username'), findsOneWidget);
      await tester.enterText(find.byType(EditableText), 'ana');
      await tester.pump();
      expect(find.text('Username'), findsNothing);
      expect(controller.text, 'ana');
    });

    testWidgets('shows the text it already holds, and no hint', (tester) async {
      await _pump(tester, text: 'ana');
      expect(find.text('ana'), findsOneWidget);
      expect(find.text('Username'), findsNothing);
    });

    testWidgets('tells of every change and of the action key', (tester) async {
      final changes = <String>[];
      final submitted = <String>[];
      await _pump(tester, changes: changes, submitted: submitted);
      await tester.enterText(find.byType(EditableText), 'an');
      await tester.enterText(find.byType(EditableText), 'ana');
      await tester.testTextInput.receiveAction(TextInputAction.done);
      expect(changes, <String>['an', 'ana']);
      expect(submitted, <String>['ana']);
    });

    testWidgets('draws the accent outline only while it has focus', (
      tester,
    ) async {
      await _pump(tester);
      Border border() =>
          ((tester
                          .widget<Container>(
                            find
                                .descendant(
                                  of: find.byType(KansoTextInput),
                                  matching: find.byType(Container),
                                )
                                .first,
                          )
                          .decoration!
                      as BoxDecoration)
                  .border!)
              as Border;
      final quiet = border().top.color;
      await tester.tap(find.byType(KansoTextInput));
      await tester.pump();
      expect(border().top.color, isNot(quiet));
      expect(border().top.width, 1.5);
    });

    testWidgets('does not take input while it is not enabled', (tester) async {
      final controller = await _pump(tester, enabled: false);
      expect(
        tester.widget<EditableText>(find.byType(EditableText)).readOnly,
        isTrue,
      );
      expect(controller.text, isEmpty);
    });

    testWidgets('is named to a screen reader by its hint', (tester) async {
      final handle = tester.ensureSemantics();
      await _pump(tester);
      expect(find.bySemanticsLabel('Username'), findsWidgets);
      handle.dispose();
    });
  });

  group('KansoTextInput with a focus node of its caller', () {
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
