// Kanso — agent_card_view.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/agent_card.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_switch.dart';

/// One change, full screen: what it is, warnings about it, and its text or
/// diff. A merge has a switch to keep the notes it combines.
class AgentCardView extends StatelessWidget {
  const AgentCardView({
    required this.card,
    required this.ticked,
    required this.keep,
    required this.onKeep,
    super.key,
  });

  final AgentCard card;
  final bool ticked;
  final bool keep;
  final ValueChanged<bool> onKeep;

  String _kind(BuildContext context) {
    final l10n = context.l10n;
    return switch (card.kind) {
      AgentCardKind.create => l10n.agentReviewKindCreate,
      AgentCardKind.edit => l10n.agentReviewKindEdit,
      AgentCardKind.move => l10n.agentReviewKindMove,
      AgentCardKind.merge => l10n.agentReviewKindMerge,
    };
  }

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final type = context.typography;
    final l10n = context.l10n;
    Widget warning(String text) => Padding(
      padding: const EdgeInsets.only(top: KansoSpacing.xs),
      child: Text(text, style: type.meta.copyWith(color: colors.danger)),
    );
    return ListView(
      padding: const EdgeInsets.all(KansoSpacing.md),
      children: <Widget>[
        Text(
          _kind(context),
          style: type.meta.copyWith(color: colors.textMuted),
        ),
        Text(
          card.title.trim().isEmpty ? '—' : card.title.trim(),
          style: type.body.copyWith(fontWeight: FontWeight.w600),
        ),
        if (card.book.isNotEmpty)
          Text(card.book, style: type.meta.copyWith(color: colors.textMuted)),
        if (card.outside) warning(l10n.agentReviewOutside),
        if (card.published) warning(l10n.agentReviewPublished),
        if (card.changedSince) warning(l10n.agentReviewChangedSince),
        const SizedBox(height: KansoSpacing.sm),
        Container(
          padding: const EdgeInsets.all(KansoSpacing.sm),
          decoration: BoxDecoration(
            color: colors.fill,
            borderRadius: BorderRadius.circular(10),
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: <Widget>[
              for (final line in card.lines) _Line(line: line),
            ],
          ),
        ),
        if (card.kind == AgentCardKind.merge) ...<Widget>[
          const SizedBox(height: KansoSpacing.md),
          Row(
            children: <Widget>[
              Expanded(
                child: Text(l10n.agentReviewKeep, style: type.body),
              ),
              KansoSwitch(value: keep, onChanged: onKeep),
            ],
          ),
          Text(
            keep ? l10n.agentReviewKept : l10n.agentReviewTrashed,
            style: type.meta.copyWith(color: colors.textMuted),
          ),
        ],
      ],
    );
  }
}

class _Line extends StatelessWidget {
  const _Line({required this.line});

  final String line;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final added = line.startsWith('+ ');
    final removed = line.startsWith('- ');
    final text = added || removed ? line.substring(2) : line;
    return Container(
      width: double.infinity,
      color: added
          ? colors.success.withValues(alpha: 0.14)
          : removed
          ? colors.danger.withValues(alpha: 0.14)
          : null,
      padding: const EdgeInsets.symmetric(vertical: 1),
      child: Text(
        '${added
            ? '+ '
            : removed
            ? '− '
            : ''}$text',
        style: context.typography.body.copyWith(
          color: removed ? colors.danger : colors.text,
        ),
      ),
    );
  }
}
