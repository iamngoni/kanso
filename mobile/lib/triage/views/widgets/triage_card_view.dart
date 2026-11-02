// Kanso — triage_card_view.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/triage_types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../models/data/triage_card_state.dart';
import '../../state/triage/triage_bloc.dart';
import 'triage_chip_tile.dart';

/// One Inbox note: its title and first lines, and its chips by field. File
/// writes what is accepted; Skip hides the card; nothing else changes the
/// note.
class TriageCardView extends StatelessWidget {
  const TriageCardView({required this.card, super.key});

  final TriageCardState card;

  static const List<TriageField> _order = <TriageField>[
    TriageField.notebook,
    TriageField.tag,
    TriageField.title,
    TriageField.link,
  ];

  String _heading(BuildContext context, TriageField field) {
    final l10n = context.l10n;
    return switch (field) {
      TriageField.notebook => l10n.triageFieldNotebook,
      TriageField.tag => l10n.triageFieldTags,
      TriageField.title => l10n.triageFieldTitle,
      TriageField.link => l10n.triageFieldLinks,
    };
  }

  Future<void> _edit(BuildContext context, int index) async {
    final bloc = context.read<TriageBloc>();
    final l10n = context.l10n;
    final controller = TextEditingController(
      text: card.chips[index].chip.label,
    );
    final text = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        content: TextField(
          controller: controller,
          autofocus: true,
          decoration: InputDecoration(hintText: l10n.triageEditHint),
          onSubmitted: (value) => Navigator.of(context).pop(value),
        ),
        actions: <Widget>[
          TextButton(
            onPressed: () => Navigator.of(context).pop(controller.text),
            child: Text(l10n.triageEditUse),
          ),
        ],
      ),
    );
    controller.dispose();
    if (text != null && text.trim().isNotEmpty) {
      bloc.add(TriageEvent.chipEdited(card.noteId, index, text));
    }
  }

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final bloc = context.read<TriageBloc>();
    final title = card.title.trim().isEmpty ? '…' : card.title;
    final meta = context.typography.meta.copyWith(
      color: colors.textSecondary,
    );
    return Container(
      margin: const EdgeInsets.symmetric(
        horizontal: KansoSpacing.md,
        vertical: KansoSpacing.xs,
      ),
      padding: const EdgeInsets.all(KansoSpacing.md),
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(16),
        border: Border.all(color: colors.divider),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: <Widget>[
          Text(title, style: context.typography.body),
          if (card.snippet.isNotEmpty) ...<Widget>[
            const SizedBox(height: KansoSpacing.xxs),
            Text(
              card.snippet,
              maxLines: 3,
              overflow: TextOverflow.ellipsis,
              style: meta,
            ),
          ],
          if (card.changed) ...<Widget>[
            const SizedBox(height: KansoSpacing.sm),
            Text(
              l10n.triageChanged,
              style: meta.copyWith(color: colors.danger),
            ),
          ],
          for (final field in _order)
            if (card.chips.any((c) => c.chip.field == field)) ...<Widget>[
              const SizedBox(height: KansoSpacing.md),
              Text(_heading(context, field), style: meta),
              for (var i = 0; i < card.chips.length; i++)
                if (card.chips[i].chip.field == field) ...<Widget>[
                  const SizedBox(height: KansoSpacing.xs),
                  TriageChipTile(
                    state: card.chips[i],
                    onAccept: () =>
                        bloc.add(TriageEvent.chipAccepted(card.noteId, i)),
                    onReject: () =>
                        bloc.add(TriageEvent.chipRejected(card.noteId, i)),
                    onEdit: () => _edit(context, i),
                  ),
                  if (card.chips[i].chip.because.isNotEmpty)
                    Padding(
                      padding: const EdgeInsets.only(top: KansoSpacing.xxs),
                      child: Text(
                        l10n.triageBecause(
                          card.chips[i].chip.because
                              .map((n) => n.title)
                              .join(', '),
                        ),
                        style: meta,
                      ),
                    ),
                ],
            ],
          if (card.chips.isEmpty) ...<Widget>[
            const SizedBox(height: KansoSpacing.md),
            Text(
              switch (card.missing) {
                TriageMissing.relatedOff => l10n.triageRelatedOff,
                TriageMissing.notRead => l10n.triageNotRead,
                _ => l10n.triageNoSuggestions,
              },
              style: meta,
            ),
            if (card.missing == TriageMissing.relatedOff) ...<Widget>[
              const SizedBox(height: KansoSpacing.sm),
              KansoButton(
                label: l10n.triageTurnOn,
                expanded: false,
                compact: true,
                onPressed: () => bloc.add(const TriageEvent.relatedTurnedOn()),
              ),
            ],
          ],
          if (card.aiError != null) ...<Widget>[
            const SizedBox(height: KansoSpacing.sm),
            Text(l10n.triageAiUnavailable(card.aiError!), style: meta),
          ],
          const SizedBox(height: KansoSpacing.md),
          Row(
            children: <Widget>[
              KansoButton(
                label: l10n.triageSkip,
                variant: KansoButtonVariant.secondary,
                expanded: false,
                compact: true,
                onPressed: () => bloc.add(TriageEvent.cardSkipped(card.noteId)),
              ),
              const SizedBox(width: KansoSpacing.xs),
              if (card.chips.isNotEmpty)
                KansoButton(
                  label: l10n.triageAcceptAll,
                  variant: KansoButtonVariant.secondary,
                  expanded: false,
                  compact: true,
                  onPressed: () =>
                      bloc.add(TriageEvent.cardAcceptedAll(card.noteId)),
                ),
              const Spacer(),
              KansoButton(
                label: l10n.triageFile,
                expanded: false,
                compact: true,
                onPressed: card.hasAccepted && !card.changed
                    ? () => bloc.add(TriageEvent.cardFiled(card.noteId))
                    : null,
              ),
            ],
          ),
        ],
      ),
    );
  }
}
