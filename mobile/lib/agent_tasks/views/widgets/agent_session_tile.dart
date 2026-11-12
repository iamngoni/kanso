// Kanso — agent_session_tile.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/agent_session.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../utils/session_text.dart';
import 'agent_status_dot.dart';

/// One session in the list: a status dot, the goal, "agent · status · when"
/// under it, an unread dot when there is news, and Stop or Review when
/// either applies. Tapping the row opens the session.
class AgentSessionTile extends StatelessWidget {
  const AgentSessionTile({
    required this.session,
    required this.nowMs,
    required this.onOpen,
    required this.onStop,
    required this.onReview,
    super.key,
  });

  final AgentSession session;

  /// The time the "when" is counted from.
  final int nowMs;
  final VoidCallback onOpen;
  final VoidCallback onStop;
  final VoidCallback onReview;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final type = context.typography;
    final l10n = context.l10n;
    final unread = session.unread;
    return KansoPressable(
      onTap: onOpen,
      child: Container(
        constraints: const BoxConstraints(minHeight: KansoSizes.touchTarget),
        padding: const EdgeInsets.fromLTRB(
          KansoSpacing.sm,
          KansoSpacing.sm,
          KansoSpacing.md,
          KansoSpacing.sm,
        ),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: <Widget>[
            Padding(
              padding: const EdgeInsets.only(top: KansoSpacing.xxs),
              child: AgentStatusDot(dot: session.dot),
            ),
            const SizedBox(width: KansoSpacing.xxs),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: <Widget>[
                  Text(
                    session.title,
                    maxLines: 2,
                    overflow: TextOverflow.ellipsis,
                    style: type.listTitle.copyWith(
                      fontWeight: unread ? FontWeight.w700 : FontWeight.w600,
                      color: colors.text,
                    ),
                  ),
                  const SizedBox(height: 2),
                  Text(
                    sessionLine(l10n, session, nowMs),
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: type.meta.copyWith(color: colors.textMuted),
                  ),
                  if (session.canStop || session.canReview) ...<Widget>[
                    const SizedBox(height: KansoSpacing.xs),
                    Wrap(
                      spacing: KansoSpacing.xs,
                      runSpacing: KansoSpacing.xs,
                      children: <Widget>[
                        if (session.canStop)
                          KansoButton(
                            label: l10n.agentTaskStop,
                            variant: KansoButtonVariant.secondary,
                            compact: true,
                            expanded: false,
                            onPressed: onStop,
                          ),
                        if (session.canReview)
                          KansoButton(
                            label: l10n.agentTaskReview,
                            compact: true,
                            expanded: false,
                            onPressed: onReview,
                          ),
                      ],
                    ),
                  ],
                ],
              ),
            ),
            if (unread) ...<Widget>[
              const SizedBox(width: KansoSpacing.xs),
              Padding(
                padding: const EdgeInsets.only(top: KansoSpacing.xs),
                child: Semantics(
                  label: l10n.agentSessionUnread,
                  child: Container(
                    key: const ValueKey<String>('agent-session-unread'),
                    width: KansoSizes.dot,
                    height: KansoSizes.dot,
                    decoration: BoxDecoration(
                      color: colors.accent,
                      shape: BoxShape.circle,
                    ),
                  ),
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }
}
