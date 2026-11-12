// Kanso — agent_transcript_view.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../ask/utils/open_answer_link.dart';
import '../../../ask/views/widgets/answer_view.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/agent_session.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';

/// Tool calls in a row that fold into one `Used 6 tools` line.
const int toolFoldAt = 3;

/// What a transcript is made of, top to bottom: the person's messages as
/// bubbles, the agent's as Markdown, thinking and tools in quiet rows, each
/// round's proposal as a card, and notices in muted type.
class AgentTranscriptView extends StatelessWidget {
  const AgentTranscriptView({
    required this.items,
    required this.agent,
    required this.canReview,
    required this.canUndo,
    required this.onReview,
    required this.onUndo,
    this.controller,
    super.key,
  });

  final List<AgentTranscriptItem> items;
  final String agent;

  /// Whether the changes waiting for review can be opened.
  final bool canReview;

  /// Whether the last apply can be taken back.
  final bool canUndo;
  final VoidCallback onReview;
  final VoidCallback onUndo;
  final ScrollController? controller;

  /// The items as blocks: three or more tool calls in a row become one.
  static List<Object> blocksOf(List<AgentTranscriptItem> items) {
    final blocks = <Object>[];
    var run = <AgentToolItem>[];
    void flush() {
      if (run.length >= toolFoldAt) {
        blocks.add(List<AgentToolItem>.unmodifiable(run));
      } else {
        blocks.addAll(run);
      }
      run = <AgentToolItem>[];
    }

    for (final item in items) {
      if (item is AgentToolItem) {
        run.add(item);
      } else {
        flush();
        blocks.add(item);
      }
    }
    flush();
    return blocks;
  }

  @override
  Widget build(BuildContext context) {
    final blocks = blocksOf(items);
    // The last proposal is the one Review and Undo belong to.
    final lastProposal = blocks.lastIndexWhere((b) => b is AgentProposedItem);
    return ListView.builder(
      controller: controller,
      padding: const EdgeInsets.fromLTRB(
        KansoSpacing.md,
        KansoSpacing.md,
        KansoSpacing.md,
        KansoSpacing.lg,
      ),
      itemCount: blocks.length,
      itemBuilder: (context, index) {
        final block = blocks[index];
        return Padding(
          padding: EdgeInsets.only(top: index == 0 ? 0 : KansoSpacing.sm),
          child: switch (block) {
            AgentUserItem(:final text) => _UserBubble(text: text),
            AgentSaidItem(:final text) => _AgentMessage(
              text: text,
              agent: agent,
            ),
            AgentThoughtItem() => _ThoughtBlock(item: block),
            AgentToolItem() => _ToolRow(item: block),
            final List<AgentToolItem> tools => _ToolGroup(tools: tools),
            AgentPermissionItem() => _PermissionLine(item: block),
            AgentProposedItem() => _ProposalCard(
              item: block,
              current: index == lastProposal,
              canReview: canReview,
              canUndo: canUndo,
              onReview: onReview,
              onUndo: onUndo,
            ),
            AgentNoticeItem(:final text) => _Notice(text: text),
            _ => const SizedBox.shrink(),
          },
        );
      },
    );
  }
}

class _UserBubble extends StatelessWidget {
  const _UserBubble({required this.text});

  final String text;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return Semantics(
      label: context.l10n.agentSessionYou,
      child: Align(
        alignment: AlignmentDirectional.centerEnd,
        child: ConstrainedBox(
          constraints: BoxConstraints(
            maxWidth: MediaQuery.sizeOf(context).width * 0.82,
          ),
          child: Container(
            padding: const EdgeInsets.symmetric(
              horizontal: KansoSpacing.sm + 2,
              vertical: KansoSpacing.xs + 2,
            ),
            decoration: BoxDecoration(
              color: colors.button,
              borderRadius: const BorderRadiusDirectional.only(
                topStart: Radius.circular(KansoRadii.large),
                topEnd: Radius.circular(KansoRadii.large),
                bottomStart: Radius.circular(KansoRadii.large),
                bottomEnd: Radius.circular(KansoRadii.small / 2),
              ),
            ),
            child: Text(
              text,
              style: context.typography.body.copyWith(color: colors.buttonText),
            ),
          ),
        ),
      ),
    );
  }
}

class _AgentMessage extends StatelessWidget {
  const _AgentMessage({required this.text, required this.agent});

  final String text;
  final String agent;

  @override
  Widget build(BuildContext context) {
    return Semantics(
      label: context.l10n.agentSessionAgentSaid(agent),
      child: AnswerView(
        markdown: text,
        usedNoteIds: const <String>{},
        onOpenNote: (_) {},
        onOpenLink: openAnswerLink,
      ),
    );
  }
}

/// `Thought for 12s ▸`, open to the text.
class _ThoughtBlock extends StatefulWidget {
  const _ThoughtBlock({required this.item});

  final AgentThoughtItem item;

  @override
  State<_ThoughtBlock> createState() => _ThoughtBlockState();
}

class _ThoughtBlockState extends State<_ThoughtBlock> {
  bool open = false;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final seconds = widget.item.seconds;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: <Widget>[
        KansoPressable(
          onTap: () => setState(() => open = !open),
          child: Padding(
            padding: const EdgeInsets.symmetric(vertical: KansoSpacing.xxs),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: <Widget>[
                KansoIcon(
                  KansoIcons.sparkles,
                  size: KansoSizes.iconSmall,
                  color: colors.textMuted,
                ),
                const SizedBox(width: KansoSpacing.xs),
                Text(
                  seconds > 0
                      ? l10n.agentSessionThought(seconds)
                      : l10n.agentSessionThoughtShort,
                  style: context.typography.meta.copyWith(
                    color: colors.textMuted,
                  ),
                ),
                const SizedBox(width: KansoSpacing.xxs),
                KansoIcon(
                  open ? KansoIcons.chevronDown : KansoIcons.chevronRight,
                  size: KansoSizes.glyph,
                  color: colors.textMuted,
                ),
              ],
            ),
          ),
        ),
        if (open)
          Container(
            margin: const EdgeInsets.only(top: KansoSpacing.xxs),
            padding: const EdgeInsets.only(left: KansoSpacing.sm),
            decoration: BoxDecoration(
              border: BorderDirectional(
                start: BorderSide(color: colors.divider, width: 2),
              ),
            ),
            child: Text(
              widget.item.text,
              style: context.typography.secondary.copyWith(
                color: colors.textSecondary,
              ),
            ),
          ),
      ],
    );
  }
}

KansoIcons _toolIcon(AgentToolKind kind) => switch (kind) {
  AgentToolKind.read => KansoIcons.fileText,
  AgentToolKind.edit => KansoIcons.pencil,
  AgentToolKind.delete => KansoIcons.trash,
  AgentToolKind.move => KansoIcons.folderOpen,
  AgentToolKind.search => KansoIcons.search,
  AgentToolKind.execute => KansoIcons.terminal,
  AgentToolKind.think => KansoIcons.sparkles,
  AgentToolKind.fetch => KansoIcons.globe,
  AgentToolKind.switchMode => KansoIcons.refreshCw,
  AgentToolKind.other => KansoIcons.wrench,
};

/// One tool call: an icon by kind, the title, and how it went.
class _ToolRow extends StatelessWidget {
  const _ToolRow({required this.item});

  final AgentToolItem item;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final status = item.status;
    final (state, color) = switch (status) {
      AgentToolStatus.completed => (l10n.agentSessionToolDone, colors.success),
      AgentToolStatus.failed => (l10n.agentSessionToolFailed, colors.danger),
      _ => (l10n.agentSessionToolRunning, colors.accent),
    };
    return Semantics(
      label: '${item.title}, $state',
      excludeSemantics: true,
      child: Row(
        children: <Widget>[
          KansoIcon(
            _toolIcon(item.kind),
            size: KansoSizes.iconSmall,
            color: colors.textMuted,
          ),
          const SizedBox(width: KansoSpacing.xs),
          Expanded(
            child: Text(
              item.title,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: context.typography.secondary.copyWith(
                color: colors.textSecondary,
              ),
            ),
          ),
          const SizedBox(width: KansoSpacing.xs),
          SizedBox.square(
            dimension: KansoSizes.iconSmall,
            child: Center(
              child: switch (status) {
                AgentToolStatus.completed => KansoIcon(
                  KansoIcons.check,
                  size: KansoSizes.iconSmall,
                  color: color,
                ),
                AgentToolStatus.failed => KansoIcon(
                  KansoIcons.close,
                  size: KansoSizes.iconSmall,
                  color: color,
                ),
                _ => Container(
                  key: const ValueKey<String>('agent-tool-running'),
                  width: KansoSizes.dot - 2,
                  height: KansoSizes.dot - 2,
                  decoration: BoxDecoration(
                    color: color,
                    shape: BoxShape.circle,
                  ),
                ),
              },
            ),
          ),
        ],
      ),
    );
  }
}

/// Three or more calls in a row: `Used 6 tools ▸`.
class _ToolGroup extends StatefulWidget {
  const _ToolGroup({required this.tools});

  final List<AgentToolItem> tools;

  @override
  State<_ToolGroup> createState() => _ToolGroupState();
}

class _ToolGroupState extends State<_ToolGroup> {
  bool open = false;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final tools = widget.tools;
    // A call that is still going or failed is worth seeing without a tap.
    final failed = tools.any((t) => t.status == AgentToolStatus.failed);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: <Widget>[
        KansoPressable(
          onTap: () => setState(() => open = !open),
          child: Padding(
            padding: const EdgeInsets.symmetric(vertical: KansoSpacing.xxs),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: <Widget>[
                KansoIcon(
                  KansoIcons.wrench,
                  size: KansoSizes.iconSmall,
                  color: failed ? colors.danger : colors.textMuted,
                ),
                const SizedBox(width: KansoSpacing.xs),
                Text(
                  context.l10n.agentSessionTools(tools.length),
                  style: context.typography.meta.copyWith(
                    color: colors.textMuted,
                  ),
                ),
                const SizedBox(width: KansoSpacing.xxs),
                KansoIcon(
                  open ? KansoIcons.chevronDown : KansoIcons.chevronRight,
                  size: KansoSizes.glyph,
                  color: colors.textMuted,
                ),
              ],
            ),
          ),
        ),
        if (open)
          Container(
            margin: const EdgeInsets.only(top: KansoSpacing.xxs),
            padding: const EdgeInsets.only(left: KansoSpacing.sm),
            decoration: BoxDecoration(
              border: BorderDirectional(
                start: BorderSide(color: colors.divider, width: 2),
              ),
            ),
            child: Column(
              children: <Widget>[
                for (var i = 0; i < tools.length; i++)
                  Padding(
                    padding: EdgeInsets.only(top: i == 0 ? 0 : KansoSpacing.xs),
                    child: _ToolRow(item: tools[i]),
                  ),
              ],
            ),
          ),
      ],
    );
  }
}

/// What the person answered when the agent asked for a tool.
class _PermissionLine extends StatelessWidget {
  const _PermissionLine({required this.item});

  final AgentPermissionItem item;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final rejected = item.answer == AgentPermissionAnswer.rejected;
    final text = switch (item.answer) {
      AgentPermissionAnswer.allowedOnce => l10n.agentSessionAllowedOnce(
        item.title,
      ),
      AgentPermissionAnswer.allowedForSession =>
        l10n.agentSessionAllowedSession(item.title),
      AgentPermissionAnswer.allowedAlways => l10n.agentSessionAllowedAlways(
        item.title,
      ),
      AgentPermissionAnswer.rejected => l10n.agentSessionRejected(item.title),
    };
    return Row(
      children: <Widget>[
        KansoIcon(
          rejected ? KansoIcons.shieldAlert : KansoIcons.shieldCheck,
          size: KansoSizes.iconSmall,
          color: rejected ? colors.danger : colors.textMuted,
        ),
        const SizedBox(width: KansoSpacing.xs),
        Expanded(
          child: Text(
            text,
            maxLines: 2,
            overflow: TextOverflow.ellipsis,
            style: context.typography.meta.copyWith(color: colors.textMuted),
          ),
        ),
      ],
    );
  }
}

/// A round's proposal: `Round 2 · 3 new notes, 1 edit` with Review, then
/// `Applied …` with Undo, or `Discarded …`.
class _ProposalCard extends StatelessWidget {
  const _ProposalCard({
    required this.item,
    required this.current,
    required this.canReview,
    required this.canUndo,
    required this.onReview,
    required this.onUndo,
  });

  final AgentProposedItem item;

  /// Whether this is the last round, the one Review and Undo belong to.
  final bool current;
  final bool canReview;
  final bool canUndo;
  final VoidCallback onReview;
  final VoidCallback onUndo;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final counts = item.counts;
    final title = switch (item.state) {
      AgentRoundState.toReview => l10n.agentSessionRound(item.round, counts),
      AgentRoundState.applied => l10n.agentSessionRoundApplied(counts),
      AgentRoundState.undone => l10n.agentSessionRoundUndone(counts),
      AgentRoundState.discarded => l10n.agentSessionRoundDiscarded(counts),
      AgentRoundState.lost => l10n.agentSessionRoundLost(counts),
    };
    final action = switch (item.state) {
      AgentRoundState.toReview when current && canReview => KansoButton(
        label: l10n.agentTaskReview,
        compact: true,
        expanded: false,
        onPressed: onReview,
      ),
      AgentRoundState.applied when current && canUndo => KansoButton(
        label: l10n.agentSessionUndo,
        variant: KansoButtonVariant.secondary,
        compact: true,
        expanded: false,
        onPressed: onUndo,
      ),
      _ => null,
    };
    final settled = item.state != AgentRoundState.toReview;
    return Container(
      padding: const EdgeInsets.all(KansoSpacing.sm + 2),
      decoration: BoxDecoration(
        color: colors.fill,
        borderRadius: KansoRadii.mediumBorder,
        border: Border.all(
          color: settled
              ? Colors.transparent
              : colors.warning.withValues(alpha: 0.5),
        ),
      ),
      child: Row(
        children: <Widget>[
          KansoIcon(
            switch (item.state) {
              AgentRoundState.applied => KansoIcons.circleCheck,
              AgentRoundState.toReview => KansoIcons.listChecks,
              _ => KansoIcons.circleX,
            },
            color: switch (item.state) {
              AgentRoundState.applied => colors.success,
              AgentRoundState.toReview => colors.warning,
              _ => colors.textMuted,
            },
          ),
          const SizedBox(width: KansoSpacing.xs + 2),
          Expanded(
            child: Text(
              title,
              style: context.typography.body.copyWith(
                fontWeight: FontWeight.w600,
                color: settled ? colors.textSecondary : colors.text,
              ),
            ),
          ),
          if (action != null) ...<Widget>[
            const SizedBox(width: KansoSpacing.xs),
            action,
          ],
        ],
      ),
    );
  }
}

/// A muted line: `Stopped`, `Hit the 10-minute limit`.
class _Notice extends StatelessWidget {
  const _Notice({required this.text});

  final String text;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Text(
        text,
        textAlign: TextAlign.center,
        style: context.typography.meta.copyWith(
          color: context.colors.textMuted,
        ),
      ),
    );
  }
}
