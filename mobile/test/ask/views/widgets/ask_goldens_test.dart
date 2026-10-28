// Kanso — ask_goldens_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/ai_note_ref.dart';
import 'package:kanso/core/models/data/ask_update.dart';

import '../../../helpers/ai_screen_goldens.dart';
import '../../../helpers/fixture_ai_repository.dart';
import '../../../helpers/pump_browse_app.dart';

const _question = 'What did we decide about pricing?';
const _used = <AiNoteRef>[
  AiNoteRef(id: 'n1', title: 'Pricing'),
  AiNoteRef(id: 'n2', title: 'Q3 plan'),
  AiNoteRef(id: 'n3'),
];

Future<void> _ask(
  WidgetTester tester,
  BrowseApp app,
  FixtureAiRepository ai,
) async {
  await tester.enterText(find.byType(EditableText).last, _question);
  await tester.pump();
  await tester.tap(find.bySemanticsLabel('Ask'));
  await tester.pumpAndSettle();
}

void main() {
  defineAiGoldens(<AiScreen>[
    const AiScreen('ask_empty', at: '/ask'),
    const AiScreen('ask_off', at: '/ask', enabled: false),
    AiScreen(
      'ask_streaming',
      at: '/ask',
      interact: (tester, app, ai) async {
        await _ask(tester, app, ai);
        ai.askUpdates
          ..add(const AskUpdate.thinking())
          ..add(const AskUpdate.usedNotes(notes: _used))
          ..add(
            const AskUpdate.answer(
              text:
                  '## Pricing\n\nWe settled on **40 a month** for the team '
                  'plan, see [Pricing](kanso://open/note/n1). Two things '
                  'are still open:\n\n- annual billing, from '
                  '[Q3 plan](kanso://open/note/n2)\n- the *free* tier',
            ),
          );
        await tester.pumpAndSettle();
      },
    ),
    AiScreen(
      'ask_done_used_notes',
      at: '/ask',
      interact: (tester, app, ai) async {
        await _ask(tester, app, ai);
        ai.askUpdates.add(
          const AskUpdate.done(
            answer:
                '## Pricing\n\nWe settled on **40 a month**, see '
                '[Pricing](kanso://open/note/n1).',
            used: _used,
          ),
        );
        await tester.pumpAndSettle();
        await tester.tap(find.text('Used 3 notes'));
        await tester.pumpAndSettle();
      },
    ),
    AiScreen(
      'ask_nothing_matches',
      at: '/ask',
      interact: (tester, app, ai) async {
        await _ask(tester, app, ai);
        ai.askUpdates.add(const AskUpdate.nothingMatches());
        await tester.pumpAndSettle();
      },
    ),
    AiScreen(
      'ask_error',
      at: '/ask',
      interact: (tester, app, ai) async {
        await _ask(tester, app, ai);
        ai.askUpdates.add(
          const AskUpdate.error(
            message: 'The server at api.anthropic.com didn’t answer.',
          ),
        );
        await tester.pumpAndSettle();
      },
    ),
  ]);
}
