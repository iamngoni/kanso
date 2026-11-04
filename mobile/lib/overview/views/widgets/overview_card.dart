// Kanso — overview_card.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_pressable.dart';

/// The width a reading column keeps on a tablet.
const double overviewColumnWidth = 640;

/// The inset of an Overview row's text from the card's edge.
const double overviewRowInset = KansoSpacing.md;

/// A section of Overview: a label above a card of rows. The card is the
/// grouped surface (`elevated` in light, `list` in dark) with a 18 point
/// radius, as the settings groups are.
class OverviewSection extends StatelessWidget {
  const OverviewSection({
    required this.label,
    required this.children,
    this.count,
    this.footer,
    super.key,
  });

  final String label;
  final int? count;
  final List<Widget> children;

  /// A quiet line or a button under the rows, inside the card.
  final Widget? footer;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    return Padding(
      padding: const EdgeInsets.only(top: KansoSpacing.lg),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          Padding(
            padding: const EdgeInsets.fromLTRB(
              KansoSpacing.lg,
              0,
              KansoSpacing.md,
              KansoSpacing.xs,
            ),
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.baseline,
              textBaseline: TextBaseline.alphabetic,
              children: <Widget>[
                Flexible(
                  child: Text(
                    label,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: typography.sectionLabel.copyWith(
                      color: colors.textSecondary,
                    ),
                  ),
                ),
                if (count != null && count! > 0) ...<Widget>[
                  const SizedBox(width: KansoSpacing.xs),
                  Text(
                    '$count',
                    style: typography.meta.copyWith(color: colors.textMuted),
                  ),
                ],
              ],
            ),
          ),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: KansoSpacing.md),
            child: DecoratedBox(
              decoration: BoxDecoration(
                color: context.groupedSurface,
                borderRadius: KansoRadii.largeBorder,
              ),
              child: ClipRRect(
                borderRadius: KansoRadii.largeBorder,
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: <Widget>[
                    for (var i = 0; i < children.length; i++) ...<Widget>[
                      if (i > 0)
                        Divider(
                          height: 1,
                          thickness: 0.5,
                          indent: overviewRowInset,
                          color: colors.divider,
                        ),
                      children[i],
                    ],
                    ?footer,
                  ],
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

/// A "N more" or "See all" line at the foot of a card.
class OverviewMoreLine extends StatelessWidget {
  const OverviewMoreLine({required this.label, required this.onTap, super.key});

  final String label;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return KansoPressable(
      onTap: onTap,
      child: Padding(
        padding: const EdgeInsets.symmetric(
          horizontal: overviewRowInset,
          vertical: KansoSpacing.sm,
        ),
        child: Text(
          label,
          style: context.typography.label.copyWith(color: colors.link),
        ),
      ),
    );
  }
}
