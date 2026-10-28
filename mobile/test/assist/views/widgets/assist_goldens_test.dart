// Kanso — assist_goldens_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/assist/views/widgets/show_assist_sheet.dart';
import 'package:kanso/core/models/data/assist_update.dart';

import '../../../helpers/ai_screen_goldens.dart';
import '../../../helpers/fixture_ai_repository.dart';
import '../../../helpers/pump_browse_app.dart';

Future<void> _open(
  WidgetTester tester,
  BrowseApp app,
  FixtureAiRepository ai,
) async {
  final context = tester.element(find.byType(Scaffold).first);
  unawaited(
    showAssistSheet(
      context,
      noteId: 'n1',
      selection: 'we ship on friday, probably',
      before: 'Plan.',
      onApply: (placement, text) => true,
    ),
  );
  await tester.pumpAndSettle();
  await tester.tap(find.text('Improve Writing'));
  await tester.pumpAndSettle();
}

void main() {
  defineAiGoldens(<AiScreen>[
    AiScreen(
      'assist_actions',
      at: '/notes',
      interact: (tester, app, ai) async {
        final context = tester.element(find.byType(Scaffold).first);
        unawaited(
          showAssistSheet(
            context,
            noteId: 'n1',
            selection: 'we ship on friday, probably',
            onApply: (placement, text) => true,
          ),
        );
        await tester.pumpAndSettle();
      },
    ),
    AiScreen(
      'assist_streaming',
      at: '/notes',
      interact: (tester, app, ai) async {
        await _open(tester, app, ai);
        ai.assistUpdates.add(
          const AssistUpdate.text(text: 'We ship on Friday, most likely'),
        );
        await tester.pumpAndSettle();
      },
    ),
    AiScreen(
      'assist_ready',
      at: '/notes',
      interact: (tester, app, ai) async {
        await _open(tester, app, ai);
        ai.assistUpdates.add(
          const AssistUpdate.done(
            suggestion: 'We will most likely ship on Friday.',
          ),
        );
        await tester.pumpAndSettle();
      },
    ),
    AiScreen(
      'assist_kept_out',
      at: '/notes',
      interact: (tester, app, ai) async {
        await _open(tester, app, ai);
        ai.assistUpdates.add(const AssistUpdate.keptOut());
        await tester.pumpAndSettle();
      },
    ),
  ]);
}
