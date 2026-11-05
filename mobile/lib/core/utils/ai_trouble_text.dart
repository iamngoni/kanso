// Kanso — ai_trouble_text.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../gen/l10n/app_localizations.dart';

/// What a model failure is, as far as its text tells. The same reading as
/// the desktop's (`kanso_ai::classify_trouble`); both suites read
/// `crates/kanso-ai/tests/trouble_corpus.json`, so the two cannot drift.
enum AiTrouble {
  missingKey('missing_key'),
  keyRejected('key_rejected'),
  missingModel('missing_model'),
  missingUrl('missing_url'),
  noAgent('no_agent'),
  agentNotInstalled('agent_not_installed'),
  agentNeedsNode('agent_needs_node'),
  noTools('no_tools'),
  busy('busy'),
  slow('slow'),
  modelNotFound('model_not_found'),
  serverDown('server_down'),
  unreachable('unreachable'),
  unreachableOllama('unreachable_ollama'),
  unreachableLmStudio('unreachable_lm_studio'),
  notSetUp('not_set_up'),
  other('other');

  const AiTrouble(this.key);

  /// The name both platforms' tests share.
  final String key;
}

/// Whether [text] has HTTP status [code] as a number of its own: "status
/// 401", "HTTP 401:", "(401)", not the 401 inside "14010" or "a401b".
bool _hasStatusCode(String text, String code) {
  bool edge(String? c) =>
      c == null || !RegExp(r'[\p{L}\p{N}_]', unicode: true).hasMatch(c);
  var from = 0;
  while (true) {
    final at = text.indexOf(code, from);
    if (at < 0) {
      return false;
    }
    final before = at == 0 ? null : text.substring(at - 1, at);
    final end = at + code.length;
    final after = end >= text.length ? null : text.substring(end, end + 1);
    if (edge(before) && edge(after)) {
      return true;
    }
    from = at + 1;
  }
}

/// What [raw] (a model error's text) means.
AiTrouble classifyAiTrouble(String raw) {
  final lower = raw.toLowerCase();
  bool has(List<String> words) => words.any(lower.contains);
  bool code(List<String> codes) => codes.any((c) => _hasStatusCode(lower, c));
  if (has(<String>[
    'add an api key',
    'no api key',
    'missing api key',
    'api key is missing',
    'api key not set',
  ])) {
    return AiTrouble.missingKey;
  }
  if (has(<String>['set a model in settings'])) {
    return AiTrouble.missingModel;
  }
  if (has(<String>["set the server's address"])) {
    return AiTrouble.missingUrl;
  }
  if (has(<String>["set the agent's command", 'no agent'])) {
    return AiTrouble.noAgent;
  }
  if (has(<String>["can't use tools"])) {
    return AiTrouble.noTools;
  }
  if (has(<String>['needs node'])) {
    return AiTrouble.agentNeedsNode;
  }
  if (has(<String>[
    "isn't installed",
    'on this computer. settings',
    'old claude code adapter',
  ])) {
    return AiTrouble.agentNotInstalled;
  }
  if (code(<String>['401', '403']) ||
      has(<String>[
        'unauthori',
        'api key',
        'invalid key',
        'forbidden',
        'authentication',
      ])) {
    return AiTrouble.keyRejected;
  }
  if (code(<String>['429']) ||
      has(<String>['rate limit', 'quota', 'too many requests'])) {
    return AiTrouble.busy;
  }
  if (has(<String>['timed out', 'timeout', "didn't answer in time"])) {
    return AiTrouble.slow;
  }
  if (code(<String>['404']) ||
      has(<String>[
        'not found',
        'no such model',
        'unknown model',
        'model_not_found',
      ])) {
    return AiTrouble.modelNotFound;
  }
  if (List<int>.generate(
        100,
        (i) => 500 + i,
      ).any((c) => _hasStatusCode(lower, '$c')) ||
      has(<String>['overloaded', 'bad gateway'])) {
    return AiTrouble.serverDown;
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
    "can't reach",
  ])) {
    if (lower.contains('ollama')) {
      return AiTrouble.unreachableOllama;
    }
    if (lower.contains('lm studio') || lower.contains('lmstudio')) {
      return AiTrouble.unreachableLmStudio;
    }
    return AiTrouble.unreachable;
  }
  if (has(<String>[
    'not set up',
    'not configured',
    'no model',
    'no provider',
    'ai is off',
  ])) {
    return AiTrouble.notSetUp;
  }
  return AiTrouble.other;
}

/// The words for [trouble]: the cause, then what to do, the same as the
/// desktop's title and detail.
String aiTroubleWords(AiTrouble trouble, AppLocalizations l10n) {
  switch (trouble) {
    case AiTrouble.missingKey:
      return l10n.aiTroubleMissingKey;
    case AiTrouble.keyRejected:
      return l10n.aiTroubleKey;
    case AiTrouble.missingModel:
      return l10n.aiTroubleMissingModel;
    case AiTrouble.missingUrl:
      return l10n.aiTroubleMissingUrl;
    case AiTrouble.noAgent:
      return l10n.aiTroubleNoAgent;
    case AiTrouble.agentNotInstalled:
      return l10n.aiTroubleAgentNotInstalled;
    case AiTrouble.agentNeedsNode:
      return l10n.aiTroubleAgentNeedsNode;
    case AiTrouble.noTools:
      return l10n.aiTroubleNoTools;
    case AiTrouble.busy:
      return l10n.aiTroubleBusy;
    case AiTrouble.slow:
      return l10n.aiTroubleSlow;
    case AiTrouble.modelNotFound:
      return l10n.aiTroubleMissing;
    case AiTrouble.serverDown:
      return l10n.aiTroubleServerDown;
    case AiTrouble.unreachable:
      return l10n.aiTroubleReach;
    case AiTrouble.unreachableOllama:
      return l10n.aiTroubleReachLocal('Ollama');
    case AiTrouble.unreachableLmStudio:
      return l10n.aiTroubleReachLocal('LM Studio');
    case AiTrouble.notSetUp:
      return l10n.aiTroubleSetup;
    case AiTrouble.other:
      return l10n.aiTroubleOther;
  }
}

/// What a model failure means, in words a person can act on: never the raw
/// error (`ollama transport error: failed to call chat completions…`). The
/// same reading and wording as the desktop's (`page_chrome::ai_trouble`).
String aiTroubleText(String raw, AppLocalizations l10n) =>
    aiTroubleWords(classifyAiTrouble(raw), l10n);
