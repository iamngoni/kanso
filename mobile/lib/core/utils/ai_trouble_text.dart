// Kanso — ai_trouble_text.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../gen/l10n/app_localizations.dart';

/// What a model failure means, in words a person can act on: never the raw
/// error (`ollama transport error: failed to call chat completions…`). The
/// same reading as the desktop's (`page_chrome::ai_trouble`).
String aiTroubleText(String raw, AppLocalizations l10n) {
  final lower = raw.toLowerCase();
  bool has(List<String> words) => words.any(lower.contains);
  if (has(<String>[
    '401',
    '403',
    'unauthori',
    'api key',
    'invalid key',
    'forbidden',
  ])) {
    return l10n.aiTroubleKey;
  }
  if (has(<String>['429', 'rate limit', 'quota', 'too many requests'])) {
    return l10n.aiTroubleBusy;
  }
  if (has(<String>['timed out', 'timeout'])) {
    return l10n.aiTroubleSlow;
  }
  if (has(<String>[
    'not found',
    '404',
    'no such model',
    'unknown model',
    'model_not_found',
  ])) {
    return l10n.aiTroubleMissing;
  }
  if (has(<String>[
    'transport',
    'connect',
    'connection',
    'refused',
    'unreachable',
    'dns',
    'network',
    'failed to call',
    'error sending request',
  ])) {
    if (lower.contains('ollama')) {
      return l10n.aiTroubleReachLocal('Ollama');
    }
    if (lower.contains('lm studio') || lower.contains('lmstudio')) {
      return l10n.aiTroubleReachLocal('LM Studio');
    }
    return l10n.aiTroubleReach;
  }
  if (has(<String>[
    'not set up',
    'not configured',
    'no model',
    'no provider',
    'ai is off',
  ])) {
    return l10n.aiTroubleSetup;
  }
  return l10n.aiTroubleOther;
}
