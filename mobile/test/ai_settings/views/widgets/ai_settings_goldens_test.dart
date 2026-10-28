// Kanso — ai_settings_goldens_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/ai_test_result.dart';

import '../../../helpers/ai_screen_goldens.dart';

void main() {
  defineAiGoldens(<AiScreen>[
    const AiScreen('ai_settings_off', at: '/settings/ai', enabled: false),
    const AiScreen('ai_settings_key_saved', at: '/settings/ai', hasKey: true),
    AiScreen(
      'ai_settings_test_ok',
      at: '/settings/ai',
      hasKey: true,
      interact: (tester, app, ai) async {
        await tester.tap(find.text('Test'));
        await tester.pumpAndSettle();
      },
    ),
    AiScreen(
      'ai_settings_test_failed',
      at: '/settings/ai',
      hasKey: true,
      interact: (tester, app, ai) async {
        ai.testResult = const AiTestResult.failed(
          message: 'The key was rejected. Check it and try again.',
        );
        await tester.tap(find.text('Test'));
        await tester.pumpAndSettle();
      },
    ),
  ]);
}
