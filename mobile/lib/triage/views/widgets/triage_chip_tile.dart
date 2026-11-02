// Kanso — triage_chip_tile.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/triage_types.dart';
import '../../../core/configs/kanso_effects.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../models/data/triage_card_state.dart';

/// One suggestion: its value, and two round buttons to accept (✓) or reject
/// (✕) it. A touch and hold on the value edits it. Accepted is filled,
/// rejected is struck through and faded, untouched is outlined; an AI
/// suggestion carries a small sparkle.
class TriageChipTile extends StatelessWidget {
  const TriageChipTile({
    required this.state,
    required this.onAccept,
    required this.onReject,
    required this.onEdit,
    super.key,
  });

  final TriageChipState state;
  final VoidCallback onAccept;
  final VoidCallback onReject;
  final VoidCallback onEdit;

  /// The value as the chip shows it.
  static String labelOf(TriageChip chip) => switch (chip.field) {
    TriageField.tag => '#${chip.label}',
    TriageField.link => '[[${chip.label}]]',
    _ => chip.label,
  };

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final chip = state.chip;
    final accepted = state.mark == ChipMark.accepted;
    final rejected = state.mark == ChipMark.rejected;
    final label = labelOf(chip);
    return Semantics(
      container: true,
      label: label,
      child: Opacity(
        opacity: rejected ? 0.55 : 1,
        child: Container(
          decoration: BoxDecoration(
            color: accepted
                ? colors.accent.withValues(alpha: KansoEffects.wash)
                : null,
            borderRadius: KansoRadii.mediumBorder,
            border: Border.all(
              color: accepted ? colors.accent : colors.divider,
            ),
          ),
          padding: const EdgeInsetsDirectional.only(
            start: KansoSpacing.sm,
            end: KansoSpacing.xxs,
          ),
          child: Row(
            children: <Widget>[
              if (chip.ai) ...<Widget>[
                KansoIcon(
                  KansoIcons.sparkles,
                  size: 14,
                  color: colors.accent,
                ),
                const SizedBox(width: KansoSpacing.xxs),
              ],
              Expanded(
                child: KansoPressable(
                  onTap: onEdit,
                  onLongPress: onEdit,
                  child: Padding(
                    padding: const EdgeInsets.symmetric(
                      vertical: KansoSpacing.sm,
                    ),
                    child: Row(
                      children: <Widget>[
                        Flexible(
                          child: Text(
                            label,
                            maxLines: 2,
                            overflow: TextOverflow.ellipsis,
                            semanticsLabel: l10n.triageEditChip(label),
                            style: context.typography.body.copyWith(
                              decoration: rejected
                                  ? TextDecoration.lineThrough
                                  : null,
                              color: accepted ? colors.text : null,
                            ),
                          ),
                        ),
                        if (chip.isNew) ...<Widget>[
                          const SizedBox(width: KansoSpacing.xs),
                          Text(
                            l10n.triageNewTag,
                            style: context.typography.meta,
                          ),
                        ],
                      ],
                    ),
                  ),
                ),
              ),
              _RoundButton(
                icon: KansoIcons.check,
                semanticLabel: l10n.triageAcceptChip(label),
                active: accepted,
                activeColor: colors.accent,
                onPressed: onAccept,
              ),
              _RoundButton(
                icon: KansoIcons.close,
                semanticLabel: l10n.triageRejectChip(label),
                active: rejected,
                activeColor: colors.danger,
                onPressed: onReject,
              ),
            ],
          ),
        ),
      ),
    );
  }
}

/// A round button of a chip: filled in its colour while it is the mark.
class _RoundButton extends StatelessWidget {
  const _RoundButton({
    required this.icon,
    required this.semanticLabel,
    required this.active,
    required this.activeColor,
    required this.onPressed,
  });

  final KansoIcons icon;
  final String semanticLabel;
  final bool active;
  final Color activeColor;
  final VoidCallback onPressed;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return Semantics(
      button: true,
      label: semanticLabel,
      excludeSemantics: true,
      child: KansoPressable(
        onTap: onPressed,
        child: Padding(
          padding: const EdgeInsets.all(KansoSpacing.xxs),
          child: Container(
            width: 36,
            height: 36,
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              color: active ? activeColor : null,
              border: Border.all(
                color: active ? activeColor : colors.divider,
              ),
            ),
            child: Center(
              child: KansoIcon(
                icon,
                size: 16,
                color: active ? colors.buttonText : colors.textSecondary,
              ),
            ),
          ),
        ),
      ),
    );
  }
}
