// Kanso — ai_screen_goldens.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'ai_blocs.dart';
import 'app_golden.dart';
import 'fixture_ai_repository.dart';
import 'kanso_viewport.dart';
import 'pump_browse_app.dart';
import 'real_themes.dart';

/// One screen of the AI features, as a picture of the whole app.
class AiScreen {
  const AiScreen(
    this.name, {
    required this.at,
    this.enabled = true,
    this.hasKey = false,
    this.interact,
  });

  /// The name in the golden’s file.
  final String name;

  /// Where the app opens.
  final String at;

  /// Whether AI is on in the settings the screen starts from.
  final bool enabled;
  final bool hasKey;

  /// What is done before the picture: typing, a tap, a script of updates the
  /// core would send.
  final Future<void> Function(
    WidgetTester tester,
    BrowseApp app,
    FixtureAiRepository ai,
  )?
  interact;
}

/// Draws each of [screens] in every window of the matrix, light and dark,
/// once as iOS and once as Android, which must match.
void defineAiGoldens(List<AiScreen> screens) {
  loadRealThemes();
  for (final viewport in <KansoViewport>[
    KansoViewport.phone,
    ...KansoViewport.goldenSet,
  ]) {
    for (final brightness in Brightness.values) {
      group('${viewport.slug}, ${brightness.name}', () {
        for (final screen in screens) {
          testWidgets(screen.name, (tester) async {
            late FixtureAiRepository ai;
            await expectAppGoldenOnBothPlatforms(
              tester,
              name: '${viewport.slug}_${screen.name}_${brightness.name}',
              viewport: viewport,
              brightness: brightness,
              at: screen.at,
              ai: () {
                ai = FixtureAiRepository(
                  settings: aiSettings(
                    enabled: screen.enabled,
                    hasKey: screen.hasKey,
                  ),
                );
                addTearDown(ai.close);
                return ai;
              },
              interact: screen.interact == null
                  ? null
                  : (tester, app) => screen.interact!(tester, app, ai),
            );
          });
        }
      });
    }
  }
}
