// Kanso — resurface_item.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';

/// A past note about the paragraph being written: its title, where and when
/// it was written, the passage that matched (as plain text, never Markdown),
/// "You wrote about this before" when it is close and old, and the actions
/// Link and Not related.
class ResurfaceItem extends StatelessWidget {
  const ResurfaceItem({
    required this.row,
    required this.onOpen,
    required this.onNotRelated,
    this.onLink,
    super.key,
  });

  final ResurfaceRow row;
  final VoidCallback onOpen;
  final VoidCallback? onLink;
  final VoidCallback onNotRelated;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    final place = row.book.isEmpty
        ? row.when
        : l10n.resurfaceNoteMeta(row.book, row.when);
    return InkWell(
      onTap: onOpen,
      child: Padding(
        padding: const EdgeInsets.symmetric(
          horizontal: KansoSpacing.md,
          vertical: KansoSpacing.sm,
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: <Widget>[
            Text(
              row.title.isEmpty ? l10n.noteUntitled : row.title,
              style: typography.body,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
            ),
            const SizedBox(height: KansoSpacing.xxs),
            Text(
              place,
              style: typography.meta.copyWith(color: colors.textMuted),
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
            ),
            if (row.snippet.isNotEmpty) ...<Widget>[
              const SizedBox(height: KansoSpacing.xxs),
              Text(
                row.snippet,
                style: typography.secondary.copyWith(
                  color: colors.textSecondary,
                ),
                maxLines: 3,
                overflow: TextOverflow.ellipsis,
              ),
            ],
            if (row.before case final before?) ...<Widget>[
              const SizedBox(height: KansoSpacing.xxs),
              Text(
                before,
                style: typography.meta.copyWith(color: colors.link),
              ),
            ],
            const SizedBox(height: KansoSpacing.xs),
            Row(
              children: <Widget>[
                if (onLink != null) ...<Widget>[
                  KansoButton(
                    label: l10n.resurfaceLink,
                    variant: KansoButtonVariant.secondary,
                    compact: true,
                    expanded: false,
                    onPressed: onLink,
                  ),
                  const SizedBox(width: KansoSpacing.xs),
                ],
                KansoButton(
                  label: l10n.resurfaceNotRelated,
                  variant: KansoButtonVariant.quiet,
                  compact: true,
                  expanded: false,
                  onPressed: onNotRelated,
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
