// Kanso — flashcards_text_test.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/flashcards/utils/flashcards_text.dart';
import 'package:kanso/gen/l10n/app_localizations_en.dart';

void main() {
  final l10n = AppLocalizationsEn();

  test('a plain message from the core is shown as it is', () {
    const plain =
        'None of these notes can be sent to AI. Locked notes and notes '
        'tagged #ai-exclude are never sent.';
    expect(FlashcardsText.failure(plain, l10n), plain);
  });

  test('a model error is never shown raw', () {
    const raw = 'ollama transport error: failed to call chat completions';
    final shown = FlashcardsText.failure(raw, l10n);
    expect(shown, isNot(contains('transport')));
    expect(shown, l10n.aiTroubleReachLocalModel);
    expect(
      FlashcardsText.failure('HTTP 401 unauthorized', l10n),
      l10n.aiTroubleKey,
    );
  });

  test('the picker names follow the bridge enums in order', () {
    expect(FlashcardsText.shapes(l10n).length, 5);
    expect(FlashcardsText.lengths(l10n).length, 3);
    expect(FlashcardsText.grades(l10n), <String>[
      'Again',
      'Hard',
      'Good',
      'Easy',
    ]);
  });
}
