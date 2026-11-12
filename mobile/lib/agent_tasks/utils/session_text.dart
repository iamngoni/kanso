// Kanso — session_text.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../core/models/data/agent_session.dart';
import '../../core/models/data/agent_task.dart';
import '../../gen/l10n/app_localizations.dart';

/// How long ago [thenMs] was at [nowMs], in a few words: `now`, `2m ago`,
/// `3h ago`, `yesterday`, `4d ago`. The same steps the desktop's list uses.
String sessionWhen(AppLocalizations l10n, int nowMs, int thenMs) {
  final secs = (nowMs - thenMs).clamp(0, 1 << 62) ~/ 1000;
  if (secs < 60) {
    return l10n.agentSessionWhenNow;
  }
  if (secs < 3600) {
    return l10n.agentSessionWhenMinutes(secs ~/ 60);
  }
  if (secs < 86400) {
    return l10n.agentSessionWhenHours(secs ~/ 3600);
  }
  if (secs < 172800) {
    return l10n.agentSessionWhenYesterday;
  }
  return l10n.agentSessionWhenDays(secs ~/ 86400);
}

/// How a session stands, in a few words: `Working…`, `4 changes to review`.
String sessionStatus(AppLocalizations l10n, AgentSession session) {
  return switch (session.phase) {
    AgentTaskPhase.waiting => l10n.agentTaskWaiting,
    AgentTaskPhase.running => l10n.agentTaskWorking,
    AgentTaskPhase.asking => l10n.agentSessionAsking,
    AgentTaskPhase.ready => l10n.agentTaskReady(session.changes),
    AgentTaskPhase.nothing => l10n.agentSessionDone,
    AgentTaskPhase.failed =>
      session.changes > 0
          ? l10n.agentTaskReady(session.changes)
          : l10n.agentSessionFailed,
    AgentTaskPhase.applied => l10n.agentTaskApplied(
      int.tryParse(session.detail) ?? session.changes,
    ),
    AgentTaskPhase.undone => l10n.agentTaskUndone,
    AgentTaskPhase.discarded => l10n.agentSessionDiscarded,
  };
}

/// The line under a title: `Kanso AI · Working… · now`.
String sessionLine(AppLocalizations l10n, AgentSession session, int nowMs) {
  return l10n.agentSessionLine(
    session.agent,
    sessionStatus(l10n, session),
    sessionWhen(l10n, nowMs, session.lastActivityMs),
  );
}
