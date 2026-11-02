// Kanso — agent_trouble.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../core/utils/ai_trouble_text.dart';
import '../../gen/l10n/app_localizations.dart';

/// What to tell the person when an agent task didn't start, from the
/// sentence the core gave. The core's own words are for diagnostics; the
/// screen shows these.
String agentTroubleText(AppLocalizations l10n, String message) {
  final lower = message.toLowerCase();
  if (lower.contains('agent tasks are turned off')) {
    return l10n.agentTroubleOff;
  }
  if (lower.contains('ai is turned off')) {
    return l10n.agentTroubleAiOff;
  }
  return aiTroubleText(message, l10n);
}

/// Whether [message] is one the person fixes in Settings rather than by
/// trying again.
bool agentTroubleIsSettings(String message) {
  final lower = message.toLowerCase();
  return lower.contains('turned off');
}
