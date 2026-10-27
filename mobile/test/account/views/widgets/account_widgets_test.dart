// Kanso — account_widgets_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/account/models/data/account_action_field.dart';
import 'package:kanso/account/views/widgets/account_action_sheet.dart';
import 'package:kanso/account/views/widgets/binding_question_sheet.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/views/widgets/kanso_code_display.dart';
import 'package:kanso/core/views/widgets/kanso_text_field.dart';

import '../../../helpers/pump.dart';

const BindingQuestion _both = BindingQuestion(
  title: 'This device already has notes',
  body: '3 notes on this device came from another account.',
  answers: <BindingAnswer>[BindingAnswer.add, BindingAnswer.replace],
);

/// Opens [open] from a button and gives what the sheet completes with.
Future<Future<T?>> _show<T>(
  WidgetTester tester,
  Future<T?> Function(BuildContext context) open,
) async {
  late Future<T?> result;
  await pumpKanso(
    tester,
    Builder(
      builder: (context) => TextButton(
        onPressed: () => result = open(context),
        child: const Text('open'),
      ),
    ),
  );
  await tester.tap(find.text('open'));
  await tester.pumpAndSettle();
  return result;
}

void main() {
  group('BindingQuestionSheet', () {
    testWidgets('asks the library’s question with a button for each answer', (
      tester,
    ) async {
      await _show<BindingAnswer>(
        tester,
        (context) => BindingQuestionSheet.show(context, _both),
      );
      expect(find.text('This device already has notes'), findsOneWidget);
      expect(find.text(_both.body), findsOneWidget);
      expect(find.text('Add them to the account'), findsOneWidget);
      expect(
        find.text('Replace them with the account’s notes'),
        findsOneWidget,
      );
      expect(find.text('Cancel'), findsOneWidget);
    });

    testWidgets('a sign-up is offered Add and Cancel only', (tester) async {
      await _show<BindingAnswer>(
        tester,
        (context) => BindingQuestionSheet.show(
          context,
          const BindingQuestion(
            title: 'Upload to the new account?',
            body: 'This device’s notes came from another account.',
            answers: <BindingAnswer>[BindingAnswer.add],
          ),
        ),
      );
      expect(find.text('Add them to the account'), findsOneWidget);
      expect(find.text('Replace them with the account’s notes'), findsNothing);
    });

    testWidgets('an answer completes the sheet with it', (tester) async {
      final result = await _show<BindingAnswer>(
        tester,
        (context) => BindingQuestionSheet.show(context, _both),
      );
      await tester.tap(find.text('Replace them with the account’s notes'));
      await tester.pumpAndSettle();
      expect(await result, BindingAnswer.replace);
      expect(find.text('This device already has notes'), findsNothing);
    });

    testWidgets('Cancel completes it with none', (tester) async {
      final result = await _show<BindingAnswer>(
        tester,
        (context) => BindingQuestionSheet.show(context, _both),
      );
      await tester.tap(find.text('Cancel'));
      await tester.pumpAndSettle();
      expect(await result, isNull);
    });
  });

  group('AccountActionSheet', () {
    Future<Future<List<String>?>> open(
      WidgetTester tester, {
      bool danger = false,
    }) {
      return _show<List<String>>(
        tester,
        (context) => AccountActionSheet.show(
          context,
          title: 'Delete account',
          message: 'This can’t be undone.',
          confirmLabel: 'Delete it',
          danger: danger,
          fields: const <AccountActionField>[
            AccountActionField(label: 'Current password'),
            AccountActionField(label: 'Name', obscure: false),
          ],
        ),
      );
    }

    testWidgets('the button waits until every field has something in it', (
      tester,
    ) async {
      final result = await open(tester);
      final fields = find.byType(EditableText);
      await tester.tap(find.text('Delete it'));
      await tester.pumpAndSettle();
      expect(find.text('Delete account'), findsOneWidget);

      await tester.enterText(fields.at(0), 'secret');
      await tester.pump();
      await tester.tap(find.text('Delete it'));
      await tester.pumpAndSettle();
      expect(find.text('Delete account'), findsOneWidget);

      await tester.enterText(fields.at(1), 'ana');
      await tester.pump();
      await tester.tap(find.text('Delete it'));
      await tester.pumpAndSettle();
      expect(await result, <String>['secret', 'ana']);
    });

    testWidgets('a password field hides what is typed, a name does not', (
      tester,
    ) async {
      await open(tester);
      final editable = tester
          .widgetList<EditableText>(find.byType(EditableText))
          .toList();
      expect(editable[0].obscureText, isTrue);
      expect(editable[1].obscureText, isFalse);
    });

    testWidgets('Cancel completes it with none', (tester) async {
      final result = await open(tester);
      await tester.tap(find.text('Cancel'));
      await tester.pumpAndSettle();
      expect(await result, isNull);
    });
  });

  group('KansoTextField', () {
    testWidgets('shows its label and hint, and keeps what is typed', (
      tester,
    ) async {
      final controller = TextEditingController();
      addTearDown(controller.dispose);
      await pumpKanso(
        tester,
        SizedBox(
          width: 300,
          child: KansoTextField(
            controller: controller,
            label: 'Username',
            hint: 'ana',
          ),
        ),
      );
      expect(find.text('Username'), findsOneWidget);
      expect(find.text('ana'), findsOneWidget);
      await tester.enterText(find.byType(EditableText), 'bo');
      await tester.pump();
      expect(controller.text, 'bo');
      expect(find.text('ana'), findsNothing);
    });

    testWidgets('a disabled field takes no input', (tester) async {
      final controller = TextEditingController(text: 'fixed');
      addTearDown(controller.dispose);
      await pumpKanso(
        tester,
        SizedBox(
          width: 300,
          child: KansoTextField(
            controller: controller,
            label: 'Username',
            enabled: false,
          ),
        ),
      );
      expect(
        tester.widget<EditableText>(find.byType(EditableText)).readOnly,
        isTrue,
      );
    });
  });

  group('KansoCodeDisplay', () {
    testWidgets('shows the code as given, and says it by its own label', (
      tester,
    ) async {
      await pumpKanso(
        tester,
        const SizedBox(
          width: 300,
          child: KansoCodeDisplay(
            text: '123 456',
            semanticLabel: 'Code 1 2 3 4 5 6',
            caption: 'Compare it.',
          ),
        ),
      );
      expect(find.text('123 456'), findsOneWidget);
      expect(find.text('Compare it.'), findsOneWidget);
      expect(find.bySemanticsLabel('Code 1 2 3 4 5 6'), findsOneWidget);
    });
  });
}
