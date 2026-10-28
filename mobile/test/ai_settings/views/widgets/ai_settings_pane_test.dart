// Kanso — ai_settings_pane_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/ai_settings/state/ai_settings/ai_settings_bloc.dart';
import 'package:kanso/ai_settings/views/widgets/ai_settings_pane.dart';
import 'package:kanso/core/models/data/ai_test_result.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:kanso/core/views/widgets/kanso_switch.dart';

import '../../../helpers/ai_blocs.dart';
import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/fixture_ai_repository.dart';
import '../../../helpers/pump_routed.dart';

Future<FixtureAiRepository> _pump(
  WidgetTester tester, {
  bool hasKey = false,
  bool enabled = true,
}) async {
  final ai = FixtureAiRepository(
    settings: aiSettings(enabled: enabled, hasKey: hasKey),
  );
  addTearDown(ai.close);
  final bloc = AiSettingsBloc(ai)..add(const AiSettingsEvent.started());
  closeBlocLater(bloc);
  await bloc.stream.firstWhere((s) => s.status == LoadStatus.ready);
  await pumpRouted(
    tester,
    BlocProvider<AiSettingsBloc>.value(
      value: bloc,
      child: const AiSettingsPane(),
    ),
    size: const Size(393, 1400),
  );
  return ai;
}

void main() {
  group('AiSettingsPane', () {
    testWidgets('has the switch, the provider, the fields and the notice', (
      tester,
    ) async {
      await _pump(tester);
      for (final text in <String>[
        'Use AI',
        'Provider',
        'Anthropic',
        'Address',
        'Model',
        'API key',
        'Test',
      ]) {
        expect(find.text(text), findsOneWidget, reason: text);
      }
      // The provider’s own address is the hint of the empty field.
      expect(find.text('https://api.anthropic.com'), findsOneWidget);
      expect(find.text(fixturePrivacy), findsOneWidget);
      // Region is Bedrock’s only.
      expect(find.text('Region'), findsNothing);
    });

    testWidgets('the switch turns AI on and saves it', (tester) async {
      final ai = await _pump(tester, enabled: false);
      await tester.tap(find.byType(KansoSwitch));
      await tester.pumpAndSettle();
      expect(ai.saved.single.enabled, isTrue);
    });

    testWidgets('choosing a provider from the list saves it', (tester) async {
      final ai = await _pump(tester);
      await tester.tap(find.text('Provider'));
      await tester.pumpAndSettle();
      expect(find.text('Choose a provider'), findsOneWidget);
      expect(
        find.text('Claude models, with a key from your Anthropic account.'),
        findsWidgets,
      );
      await tester.tap(find.text('Local'));
      await tester.pumpAndSettle();
      expect(find.text('Choose a provider'), findsNothing);
      expect(ai.saved.single.provider, 'ollama');
      // The form starts again from the new provider: its key is optional.
      expect(find.text('API key (optional)'), findsOneWidget);
    });

    testWidgets('a key is sent and never shown again', (tester) async {
      final ai = await _pump(tester);
      const secret = 'sk-test-0123456789';
      await tester.enterText(find.byType(EditableText).last, secret);
      await tester.pump();
      await tester.tap(find.text('Save key'));
      await tester.pumpAndSettle();
      expect(ai.keysSet, <String>[secret]);
      expect(find.text('Saved'), findsOneWidget);
      expect(find.text('Remove'), findsOneWidget);
      expect(find.textContaining(secret), findsNothing);
      expect(
        tester
            .widgetList<EditableText>(find.byType(EditableText))
            .any((field) => field.controller.text.contains(secret)),
        isFalse,
      );
      await tester.tap(find.text('Remove'));
      await tester.pumpAndSettle();
      expect(ai.keysSet.last, isEmpty);
      expect(find.text('Save key'), findsOneWidget);
    });

    testWidgets('a saved key is shown as Saved from the start', (
      tester,
    ) async {
      await _pump(tester, hasKey: true);
      expect(find.text('Saved'), findsOneWidget);
      expect(find.text('Remove'), findsOneWidget);
      expect(find.text('Paste the key'), findsNothing);
    });

    testWidgets('Test shows OK with what the model said', (tester) async {
      await _pump(tester);
      await tester.tap(find.text('Test'));
      await tester.pumpAndSettle();
      expect(find.text('Ready. The model said “OK”.'), findsOneWidget);
    });

    testWidgets('Test shows why it failed', (tester) async {
      final ai = await _pump(tester);
      ai.testResult = const AiTestResult.failed(
        message: 'The server at that address didn’t answer.',
      );
      await tester.tap(find.text('Test'));
      await tester.pumpAndSettle();
      expect(
        find.text('The server at that address didn’t answer.'),
        findsOneWidget,
      );
    });
  });
}
