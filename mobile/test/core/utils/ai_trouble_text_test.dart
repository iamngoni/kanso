// Kanso — ai_trouble_text_test.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados, StringAnys, any;
import 'package:kanso/core/utils/ai_trouble_text.dart';
import 'package:kanso/gen/l10n/app_localizations_en.dart';

import '../../helpers/property.dart';

void main() {
  final l10n = AppLocalizationsEn();

  group('aiTroubleText', () {
    test('a dead Ollama reads as a person would say it', () {
      expect(
        aiTroubleText(
          'The model reported an error: ollama transport error: failed to '
          'call chat completions for streamed message',
          l10n,
        ),
        l10n.aiTroubleReachLocal('Ollama'),
      );
    });

    test('keys, limits and time-outs each say what to do', () {
      expect(aiTroubleText('HTTP 401 Unauthorized', l10n), l10n.aiTroubleKey);
      expect(aiTroubleText('429 Too Many Requests', l10n), l10n.aiTroubleBusy);
      expect(aiTroubleText('request timed out', l10n), l10n.aiTroubleSlow);
    });

    Glados<String>(any.letterOrDigits, kansoExploreConfig()).test(
      'never shows the raw error',
      (raw) {
        final text = aiTroubleText('raw-$raw', l10n);
        expect(text, isNot(contains('raw-$raw')));
        expect(text, isNotEmpty);
      },
    );
  });
}
