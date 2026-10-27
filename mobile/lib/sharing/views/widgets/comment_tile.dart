// Kanso — comment_tile.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:clock/clock.dart';
import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/time_words.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_pressable.dart';

/// One comment as the desktop's Comments tab draws it: its author, when it
/// was written (edited, still on its way), the words it is on and what it
/// says, @mentions of people in the note standing out. A resolved one is
/// dimmed and cut to a line; an orphaned one's words are struck through.
///
/// Anyone who may comment can Resolve or Reopen it; the account's own
/// comments add Edit and Delete, which asks once. Tapping it runs
/// [onSelect].
class CommentTile extends StatefulWidget {
  const CommentTile({
    required this.comment,
    required this.onSelect,
    required this.onResolve,
    required this.onEdit,
    required this.onDelete,
    super.key,
  });

  final CommentRow comment;
  final VoidCallback onSelect;
  final ValueChanged<bool> onResolve;
  final VoidCallback onEdit;
  final VoidCallback onDelete;

  @override
  State<CommentTile> createState() => _CommentTileState();
}

class _CommentTileState extends State<CommentTile> {
  bool confirming = false;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    final comment = widget.comment;
    final orphaned = comment.placement is CommentPlacement_Orphaned;
    final quote = comment.quote.trim();
    final when = StringBuffer(
      editedLabel(l10n, comment.createdMs, now: clock.now()),
    );
    if (comment.editedMs != null) {
      when.write(l10n.commentEditedSuffix);
    }
    if (comment.pending) {
      when.write(l10n.commentSendingSuffix);
    }
    final actions = <(String, VoidCallback)>[
      if (comment.canResolve)
        (
          comment.resolved ? l10n.commentReopen : l10n.commentResolve,
          () => widget.onResolve(!comment.resolved),
        ),
      if (comment.canEdit) (l10n.commentEdit, widget.onEdit),
      if (comment.canDelete)
        (l10n.commentDelete, () => setState(() => confirming = true)),
    ];
    return Opacity(
      opacity: comment.resolved ? 0.55 : 1,
      child: KansoPressable(
        onTap: widget.onSelect,
        child: Padding(
          padding: const EdgeInsets.symmetric(
            horizontal: KansoSpacing.md,
            vertical: KansoSpacing.xs,
          ),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: <Widget>[
              Row(
                children: <Widget>[
                  // One line, so a long name gives way to the time.
                  Expanded(
                    child: Text.rich(
                      TextSpan(
                        children: <InlineSpan>[
                          TextSpan(
                            text: comment.authorName,
                            style: typography.label,
                          ),
                          const TextSpan(text: '  '),
                          TextSpan(
                            text: when.toString(),
                            style: typography.meta.copyWith(
                              color: colors.textMuted,
                            ),
                          ),
                        ],
                      ),
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                  if (comment.resolved)
                    Text(
                      l10n.commentResolvedMark,
                      style: typography.meta.copyWith(color: colors.textMuted),
                    ),
                ],
              ),
              if (quote.isNotEmpty) ...<Widget>[
                const SizedBox(height: KansoSpacing.xxs),
                Container(
                  padding: const EdgeInsetsDirectional.only(
                    start: KansoSpacing.xs,
                  ),
                  decoration: BoxDecoration(
                    border: BorderDirectional(
                      start: BorderSide(
                        color: orphaned
                            ? colors.divider
                            : colors.warning.withValues(alpha: 0.6),
                        width: 2,
                      ),
                    ),
                  ),
                  child: Text(
                    quote,
                    maxLines: orphaned ? 2 : 1,
                    overflow: TextOverflow.ellipsis,
                    style: typography.secondary.copyWith(
                      fontStyle: FontStyle.italic,
                      decoration: orphaned ? TextDecoration.lineThrough : null,
                    ),
                  ),
                ),
              ],
              const SizedBox(height: KansoSpacing.xxs),
              Text.rich(
                _body(comment, accent: colors.accent),
                maxLines: comment.resolved ? 1 : null,
                overflow: comment.resolved ? TextOverflow.ellipsis : null,
                style: typography.body,
              ),
              if (confirming) ...<Widget>[
                const SizedBox(height: KansoSpacing.xs),
                Text(
                  l10n.commentDeleteWarning,
                  style: typography.secondary.copyWith(color: colors.danger),
                ),
                const SizedBox(height: KansoSpacing.xs),
                Row(
                  children: <Widget>[
                    KansoButton(
                      label: l10n.commentDelete,
                      variant: KansoButtonVariant.danger,
                      expanded: false,
                      compact: true,
                      onPressed: widget.onDelete,
                    ),
                    const SizedBox(width: KansoSpacing.xs),
                    KansoButton(
                      label: l10n.noteListCancel,
                      variant: KansoButtonVariant.quiet,
                      expanded: false,
                      compact: true,
                      onPressed: () => setState(() => confirming = false),
                    ),
                  ],
                ),
              ] else if (actions.isNotEmpty)
                Wrap(
                  spacing: KansoSpacing.xxs,
                  children: <Widget>[
                    for (final (label, onPressed) in actions)
                      KansoButton(
                        label: label,
                        variant: KansoButtonVariant.quiet,
                        expanded: false,
                        compact: true,
                        onPressed: onPressed,
                      ),
                  ],
                ),
            ],
          ),
        ),
      ),
    );
  }
}

/// The comment's words with its @mentions of people in the note in
/// [accent]; the mentions count in UTF-16 units, as Dart's strings do.
TextSpan _body(CommentRow comment, {required Color accent}) {
  final text = comment.body;
  final spans = <InlineSpan>[];
  var at = 0;
  for (final mention in comment.mentions) {
    final start = mention.start.clamp(at, text.length);
    final end = mention.end.clamp(start, text.length);
    if (start > at) {
      spans.add(TextSpan(text: text.substring(at, start)));
    }
    if (end > start) {
      spans.add(
        TextSpan(
          text: text.substring(start, end),
          style: TextStyle(color: accent, fontWeight: FontWeight.w500),
        ),
      );
    }
    at = end;
  }
  if (at < text.length) {
    spans.add(TextSpan(text: text.substring(at)));
  }
  return TextSpan(children: spans);
}
