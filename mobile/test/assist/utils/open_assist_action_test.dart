// Kanso — open_assist_action_test.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/assist/utils/open_assist_action.dart';
import 'package:kanso/core/models/data/assist_action.dart';

import '../../helpers/ai_blocs.dart';
import '../../helpers/fake_editor_target.dart';
import '../../helpers/fixture_ai_repository.dart';
import '../../helpers/pump_browse_app.dart';

const Size _size = Size(1180, 820);

Future<(FixtureAiRepository, BuildContext)> _pump(WidgetTester tester) async {
  final ai = FixtureAiRepository(
    settings: aiSettings(enabled: true, hasKey: true),
  );
  await pumpBrowseApp(tester, _size, ai: ai);
  return (ai, tester.element(find.byType(Scaffold).first));
}

void main() {
  group('paragraphBefore', () {
    test('takes the paragraph ending at the caret', () {
      const text = 'Earlier.\n\nWe ship on Friday.';
      expect(
        paragraphBefore(text, text.length),
        const TextRange(start: 10, end: 28),
      );
    });

    test('omits whitespace at the paragraph start', () {
      const text = 'Earlier.\n\n   We ship on Friday.';
      expect(
        paragraphBefore(text, text.length),
        const TextRange(start: 13, end: 31),
      );
    });

    test('is collapsed after only whitespace', () {
      const text = 'Earlier.\n\n   ';
      expect(
        paragraphBefore(text, text.length),
        const TextRange.collapsed(13),
      );
    });
  });

  group('openAssistAction', () {
    testWidgets('a rewrite with no selection explains what is needed', (
      tester,
    ) async {
      final (ai, context) = await _pump(tester);
      final target = FakeEditorTarget('We ship on Friday.');

      await openAssistAction(context, target, action: AssistAction.improve);
      await tester.pump();

      expect(find.text('Select some text first.'), findsOneWidget);
      expect(ai.assistRequests, isEmpty);
    });

    testWidgets('Continue Writing uses the paragraph before the caret', (
      tester,
    ) async {
      final (ai, context) = await _pump(tester);
      final target = FakeEditorTarget('Earlier.\n\nWe ship on Friday.');

      unawaited(
        openAssistAction(
          context,
          target,
          action: AssistAction.continueWriting,
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('Continued'), findsOneWidget);
      expect(ai.assistRequests, hasLength(1));
      expect(ai.assistRequests.single.action, AssistAction.continueWriting);
      expect(ai.assistRequests.single.selection, 'We ship on Friday.');
      await tester.tap(find.text('Discard'));
      await tester.pumpAndSettle();
    });

    testWidgets('a direct action opens already running on the selection', (
      tester,
    ) async {
      final (ai, context) = await _pump(tester);
      final target = FakeEditorTarget(
        'Plan. We ship on Friday.',
        selection: const TextSelection(baseOffset: 6, extentOffset: 24),
      );

      unawaited(
        openAssistAction(context, target, action: AssistAction.improve),
      );
      await tester.pumpAndSettle();

      expect(find.text('Improved'), findsOneWidget);
      expect(find.text('Thinking…'), findsOneWidget);
      expect(find.text('Stop'), findsOneWidget);
      expect(ai.assistRequests, hasLength(1));
      expect(ai.assistRequests.single.action, AssistAction.improve);
      expect(ai.assistRequests.single.selection, 'We ship on Friday.');
      await tester.tap(find.text('Discard'));
      await tester.pumpAndSettle();
    });
  });
}
