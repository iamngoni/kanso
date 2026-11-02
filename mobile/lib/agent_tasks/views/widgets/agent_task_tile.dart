// Kanso — agent_task_tile.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/agent_task.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_progress.dart';
import '../../../core/views/widgets/kanso_text_field.dart';

/// One task: its goal, what it is doing, and what can be done with it.
class AgentTaskTile extends StatefulWidget {
  const AgentTaskTile({
    required this.task,
    required this.onStop,
    required this.onReview,
    required this.onDismiss,
    required this.onReply,
    super.key,
  });

  final AgentTask task;
  final VoidCallback onStop;
  final VoidCallback onReview;
  final VoidCallback onDismiss;
  final ValueChanged<String> onReply;

  @override
  State<AgentTaskTile> createState() => _AgentTaskTileState();
}

class _AgentTaskTileState extends State<AgentTaskTile> {
  final TextEditingController _reply = TextEditingController();

  @override
  void dispose() {
    _reply.dispose();
    super.dispose();
  }

  String _status(BuildContext context) {
    final l10n = context.l10n;
    final task = widget.task;
    return switch (task.phase) {
      AgentTaskPhase.waiting => l10n.agentTaskWaiting,
      AgentTaskPhase.running =>
        task.line.isEmpty ? l10n.agentTaskWorking : task.line,
      AgentTaskPhase.asking => task.detail,
      AgentTaskPhase.ready => l10n.agentTaskReady(task.changes),
      AgentTaskPhase.nothing => l10n.agentTaskNothing,
      AgentTaskPhase.failed =>
        task.changes > 0
            ? l10n.agentTaskReady(task.changes)
            : l10n.agentTroubleGeneric,
      AgentTaskPhase.applied => l10n.agentTaskApplied(
        int.tryParse(task.detail) ?? task.changes,
      ),
      AgentTaskPhase.undone => l10n.agentTaskUndone,
      AgentTaskPhase.discarded => '',
    };
  }

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final type = context.typography;
    final l10n = context.l10n;
    final task = widget.task;
    return Container(
      margin: const EdgeInsets.only(bottom: KansoSpacing.sm),
      padding: const EdgeInsets.all(KansoSpacing.md),
      decoration: BoxDecoration(
        color: colors.fill,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: <Widget>[
          Text(
            task.goal,
            maxLines: 3,
            overflow: TextOverflow.ellipsis,
            style: type.body.copyWith(fontWeight: FontWeight.w600),
          ),
          const SizedBox(height: KansoSpacing.xxs),
          Text(
            _status(context),
            maxLines: 4,
            overflow: TextOverflow.ellipsis,
            style: type.body.copyWith(color: colors.textSecondary),
          ),
          if (task.active) ...<Widget>[
            const SizedBox(height: KansoSpacing.xs),
            const KansoProgressBar(),
          ],
          if (task.notesRead > 0) ...<Widget>[
            const SizedBox(height: KansoSpacing.xxs),
            Text(
              l10n.agentTaskNotesRead(task.notesRead),
              style: type.meta.copyWith(color: colors.textMuted),
            ),
          ],
          if (task.phase == AgentTaskPhase.asking) ...<Widget>[
            const SizedBox(height: KansoSpacing.xs),
            KansoTextField(controller: _reply),
            const SizedBox(height: KansoSpacing.xs),
            KansoButton(
              label: l10n.agentTaskSend,
              compact: true,
              expanded: false,
              onPressed: () {
                final text = _reply.text.trim();
                if (text.isNotEmpty) {
                  widget.onReply(text);
                  _reply.clear();
                }
              },
            ),
          ],
          const SizedBox(height: KansoSpacing.xs),
          Wrap(
            spacing: KansoSpacing.xs,
            runSpacing: KansoSpacing.xs,
            children: <Widget>[
              if (task.active || task.phase == AgentTaskPhase.asking)
                KansoButton(
                  label: l10n.agentTaskStop,
                  variant: KansoButtonVariant.secondary,
                  compact: true,
                  expanded: false,
                  onPressed: widget.onStop,
                ),
              if (task.reviewable)
                KansoButton(
                  label: l10n.agentTaskReview,
                  compact: true,
                  expanded: false,
                  onPressed: widget.onReview,
                ),
              if (!task.active && task.phase != AgentTaskPhase.asking)
                KansoButton(
                  label: l10n.agentTaskDismiss,
                  variant: KansoButtonVariant.quiet,
                  compact: true,
                  expanded: false,
                  onPressed: widget.onDismiss,
                ),
            ],
          ),
        ],
      ),
    );
  }
}
