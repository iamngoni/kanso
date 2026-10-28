// Kanso — search_result_tile.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_lock_mark.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../lock/state/lock_status/lock_status_bloc.dart';
import 'highlighted_text.dart';

/// A search result: the note's title, its notebook and the text around the
/// match, with what matched highlighted.
///
/// A locked note shows its title (found by it), its lock and the word
/// "Locked", and nothing of its text. [open] is the note open beside the
/// results.
class SearchResultTile extends StatelessWidget {
  const SearchResultTile({
    required this.hit,
    required this.notebook,
    this.open = false,
    this.onTap,
    super.key,
  });

  final Hit hit;

  /// Where the note is, as a path (`Projects / Kanso`).
  final String notebook;
  final bool open;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    final row = hit.row;
    final untitled = row.title.trim().isEmpty;
    final unlocked =
        row.locked &&
        context.select<LockStatusBloc, bool>((lock) => lock.state.unlocked);
    return KansoPressable(
      onTap: onTap,
      child: Container(
        color: open ? colors.rowActive : null,
        padding: const EdgeInsets.symmetric(
          horizontal: KansoSpacing.md,
          vertical: KansoSpacing.sm,
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: <Widget>[
            Row(
              children: <Widget>[
                Expanded(
                  child: HighlightedText(
                    untitled ? l10n.noteUntitled : row.title,
                    ranges: untitled ? const <Range16>[] : hit.titleRanges,
                    style: typography.listTitle,
                    maxLines: 1,
                  ),
                ),
                if (row.locked) ...<Widget>[
                  const SizedBox(width: KansoSpacing.xs),
                  KansoLockMark(unlocked: unlocked),
                ],
              ],
            ),
            const SizedBox(height: KansoSpacing.xxs),
            Row(
              children: <Widget>[
                KansoIcon(
                  KansoIcons.notebook,
                  size: KansoSizes.glyph,
                  color: colors.textMuted,
                ),
                const SizedBox(width: KansoSpacing.xxs),
                Flexible(
                  child: Text(
                    notebook,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: typography.meta,
                  ),
                ),
              ],
            ),
            if (row.locked || row.snippet.isNotEmpty) ...<Widget>[
              const SizedBox(height: KansoSpacing.xs),
              if (row.locked)
                Text(
                  unlocked ? l10n.noteListUnlocked : l10n.noteListLocked,
                  style: typography.secondary,
                )
              else
                HighlightedText(
                  row.snippet,
                  ranges: hit.snippetRanges,
                  style: typography.secondary,
                  maxLines: 2,
                ),
            ],
          ],
        ),
      ),
    );
  }
}
