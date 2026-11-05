// Kanso — ai_trouble_text_test.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:convert';
import 'dart:io';

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
          'The model reported an error: ollama transport error: error '
          'sending request for url (http://localhost:11434/v1/chat/completions)',
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

    test('a missing key says it is missing and where to add one', () {
      final text = aiTroubleText('Add an API key in Settings › AI.', l10n);
      expect(text, l10n.aiTroubleMissingKey);
      expect(text, startsWith('No API key is saved.'));
      expect(text, contains('Settings › AI'));
    });

    test('a missing agent says no agent is set up and how to set one up', () {
      final text = aiTroubleText(
        "Set the agent's command in Settings › AI.",
        l10n,
      );
      expect(text, l10n.aiTroubleNoAgent);
      expect(text, startsWith('No agent is set up.'));
      expect(text, contains('Choose an agent in Settings › AI'));
    });

    test('a proxy or gateway refusal does not blame the key', () {
      final text = aiTroubleText(
        "The model's server answered 407: Proxy Authentication Required",
        l10n,
      );
      expect(text, l10n.aiTroubleBlocked);
      expect(text, isNot(l10n.aiTroubleKey));
      expect(
        aiTroubleText(
          "The model's server answered 403: <html><title>403 Forbidden",
          l10n,
        ),
        l10n.aiTroubleBlocked,
      );
    });

    test('a Local model error names no app it cannot tell', () {
      expect(
        aiTroubleText(
          'The model reported an error: ollama transport error: failed to '
          'call chat completions for streamed message',
          l10n,
        ),
        l10n.aiTroubleReachLocalModel,
      );
      expect(l10n.aiTroubleReachLocalModel, contains('Ollama or LM Studio'));
    });

    test('every kind has its own non-empty words', () {
      final seen = <String>{};
      for (final trouble in AiTrouble.values) {
        final text = aiTroubleWords(trouble, l10n);
        expect(text.trim(), isNotEmpty, reason: trouble.key);
        seen.add(text);
      }
      expect(seen.length, AiTrouble.values.length);
    });

    test('reads the desktop corpus the same way', () {
      final file = File('../crates/kanso-ai/tests/trouble_corpus.json');
      final cases = jsonDecode(file.readAsStringSync()) as List<dynamic>;
      expect(cases, isNotEmpty);
      for (final c in cases) {
        final map = c as Map<String, dynamic>;
        expect(
          classifyAiTrouble(map['raw'] as String).key,
          map['kind'],
          reason: map['raw'] as String,
        );
      }
    });

    Glados<String>(any.letterOrDigits, kansoExploreConfig()).test(
      'never shows the raw error',
      (raw) {
        final text = aiTroubleText('raw-$raw', l10n);
        expect(text, isNot(contains('raw-$raw')));
        expect(text, isNotEmpty);
      },
    );

    Glados<String>(any.letterOrDigits, kansoExploreConfig()).test(
      'no message is empty, whatever the error says',
      (raw) {
        expect(aiTroubleText(raw, l10n).trim(), isNotEmpty);
      },
    );
  });
}
