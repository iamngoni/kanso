// Kanso — new_notebook_kind_picker.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_effects.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_check_badge.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';

/// The choice between a blank notebook and a Video studio: two rounded
/// cards, each with a tinted icon tile, a title and what it sets up. The
/// chosen one has an accent outline and a check.
class NewNotebookKindPicker extends StatelessWidget {
  const NewNotebookKindPicker({
    required this.kind,
    required this.onChanged,
    super.key,
  });

  final NewNotebookKind kind;
  final ValueChanged<NewNotebookKind> onChanged;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return Semantics(
      container: true,
      label: l10n.newNotebookKindLabel,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          _KindCard(
            icon: KansoIcons.book,
            title: l10n.newNotebookKindBlank,
            hint: l10n.newNotebookKindBlankHint,
            selected: kind == NewNotebookKind.blank,
            onTap: () => onChanged(NewNotebookKind.blank),
          ),
          const SizedBox(height: KansoSpacing.xs),
          _KindCard(
            icon: KansoIcons.clapperboard,
            title: l10n.newNotebookKindStudio,
            hint: l10n.newNotebookKindStudioHint,
            selected: kind == NewNotebookKind.videoStudio,
            onTap: () => onChanged(NewNotebookKind.videoStudio),
          ),
        ],
      ),
    );
  }
}

class _KindCard extends StatelessWidget {
  const _KindCard({
    required this.icon,
    required this.title,
    required this.hint,
    required this.selected,
    required this.onTap,
  });

  final KansoIcons icon;
  final String title;
  final String hint;
  final bool selected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    return Semantics(
      button: true,
      inMutuallyExclusiveGroup: true,
      selected: selected,
      label: title,
      hint: hint,
      onTap: onTap,
      excludeSemantics: true,
      child: KansoPressable(
        onTap: onTap,
        scale: 0.98,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 160),
          curve: Curves.easeOut,
          padding: const EdgeInsets.all(KansoSpacing.sm),
          decoration: BoxDecoration(
            color: selected
                ? colors.accent.withValues(alpha: KansoEffects.quietWash)
                : colors.elevated,
            borderRadius: KansoRadii.largeBorder,
            border: Border.all(
              color: selected ? colors.accent : colors.divider,
              width: selected ? 1.5 : 1,
            ),
          ),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: <Widget>[
              Container(
                width: 36,
                height: 36,
                alignment: Alignment.center,
                decoration: BoxDecoration(
                  color: colors.accent.withValues(alpha: KansoEffects.wash),
                  borderRadius: KansoRadii.mediumBorder,
                ),
                child: KansoIcon(icon, color: colors.accent),
              ),
              const SizedBox(width: KansoSpacing.sm),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: <Widget>[
                    Text(title, style: typography.listTitle),
                    const SizedBox(height: 2),
                    Text(
                      hint,
                      style: typography.secondary.copyWith(
                        color: colors.textSecondary,
                      ),
                    ),
                  ],
                ),
              ),
              const SizedBox(width: KansoSpacing.xs),
              SizedBox(
                width: KansoCheckBadge.diameter,
                height: KansoCheckBadge.diameter,
                child: selected
                    ? const KansoCheckBadge(semanticLabel: '')
                    : DecoratedBox(
                        decoration: BoxDecoration(
                          shape: BoxShape.circle,
                          border: Border.all(color: colors.divider, width: 1.5),
                        ),
                      ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
