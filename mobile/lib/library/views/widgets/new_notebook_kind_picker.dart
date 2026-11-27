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
import '../../../core/utils/starter_tag_color.dart';
import '../../../core/views/widgets/kanso_check_badge.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_tag_chip.dart';

/// What a new notebook starts as: a rounded card for each of [starters], each
/// with a tinted icon tile, a title, what it sets up, the notebooks and tags
/// it makes. The chosen one has an accent outline and a check.
///
/// The words are the library's, the same as the desktop's: they are the
/// names of the notebooks and tags that get made. When [lockedParent] is set
/// the cards that make notes are dimmed, say why, and can't be chosen.
class NewNotebookKindPicker extends StatelessWidget {
  const NewNotebookKindPicker({
    required this.starters,
    required this.kind,
    required this.onChanged,
    this.lockedParent = false,
    super.key,
  });

  final List<StarterInfo> starters;
  final NewNotebookKind kind;
  final ValueChanged<NewNotebookKind> onChanged;

  /// The notebook is made inside a locked notebook.
  final bool lockedParent;

  @override
  Widget build(BuildContext context) {
    return Semantics(
      container: true,
      label: context.l10n.newNotebookKindLabel,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          for (final (index, starter) in starters.indexed) ...<Widget>[
            if (index > 0) const SizedBox(height: KansoSpacing.xs),
            _KindCard(
              starter: starter,
              selected: kind == starter.kind,
              disabled: lockedParent && starter.makesNotes,
              onTap: () => onChanged(starter.kind),
            ),
          ],
        ],
      ),
    );
  }
}

class _KindCard extends StatelessWidget {
  const _KindCard({
    required this.starter,
    required this.selected,
    required this.disabled,
    required this.onTap,
  });

  final StarterInfo starter;
  final bool selected;
  final bool disabled;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    final locked = context.l10n.newNotebookStarterLocked;
    final secondary = typography.secondary.copyWith(
      color: colors.textSecondary,
    );
    return Semantics(
      button: true,
      inMutuallyExclusiveGroup: true,
      enabled: !disabled,
      selected: selected,
      label: starter.label,
      hint: disabled ? '${starter.description} $locked' : starter.description,
      onTap: disabled ? null : onTap,
      excludeSemantics: true,
      child: Opacity(
        opacity: disabled ? 0.5 : 1,
        child: KansoPressable(
          onTap: disabled ? null : onTap,
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
                  child: KansoIcon(
                    KansoIcons.named(starter.icon),
                    color: colors.accent,
                  ),
                ),
                const SizedBox(width: KansoSpacing.sm),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: <Widget>[
                      Text(starter.label, style: typography.listTitle),
                      const SizedBox(height: 2),
                      Text(starter.description, style: secondary),
                      if (disabled) ...<Widget>[
                        const SizedBox(height: 2),
                        Text(locked, style: secondary),
                      ],
                      if (starter.subNotebooks.isNotEmpty) ...<Widget>[
                        const SizedBox(height: KansoSpacing.xs),
                        _LineWithIcon(
                          icon: KansoIcons.notebook,
                          child: Text(
                            starter.subNotebooks.join(' · '),
                            style: secondary,
                          ),
                        ),
                      ],
                      if (starter.tags.isNotEmpty) ...<Widget>[
                        const SizedBox(height: KansoSpacing.xs),
                        _LineWithIcon(
                          icon: KansoIcons.tag,
                          child: Wrap(
                            spacing: 4,
                            runSpacing: 4,
                            children: <Widget>[
                              for (final tag in starter.tags)
                                KansoTagChip(name: tag.name, color: tag.tint),
                            ],
                          ),
                        ),
                      ],
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
                            border: Border.all(
                              color: colors.divider,
                              width: 1.5,
                            ),
                          ),
                        ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

/// A small icon before a line: what the line lists.
class _LineWithIcon extends StatelessWidget {
  const _LineWithIcon({required this.icon, required this.child});

  final KansoIcons icon;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: <Widget>[
        Padding(
          padding: const EdgeInsets.only(top: 2),
          child: KansoIcon(
            icon,
            size: 14,
            color: context.colors.textSecondary,
          ),
        ),
        const SizedBox(width: KansoSpacing.xs),
        Expanded(child: child),
      ],
    );
  }
}
