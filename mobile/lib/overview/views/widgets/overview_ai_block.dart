// Kanso — overview_ai_block.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';

import '../../../core/bridge/api/overview_types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../utils/linked_text.dart';

/// Past this many characters a paragraph is cut short with "More".
const int overviewAiShortChars = 280;

/// The model's paragraph over the page: a quiet accent rule, the words with
/// each citation a link to its note, a warning when it named a note that
/// was not on the page, and a footnote that says who wrote it, with
/// Rewrite and Hide.
class OverviewAiBlock extends StatefulWidget {
  const OverviewAiBlock({
    required this.ai,
    required this.more,
    required this.onOpenNote,
    required this.onMore,
    required this.onRewrite,
    required this.onHide,
    super.key,
  });

  final OverviewAi ai;
  final bool more;
  final void Function(String noteId) onOpenNote;
  final VoidCallback onMore;
  final VoidCallback onRewrite;
  final VoidCallback onHide;

  @override
  State<OverviewAiBlock> createState() => _OverviewAiBlockState();
}

class _OverviewAiBlockState extends State<OverviewAiBlock> {
  List<TapGestureRecognizer> recognizers = <TapGestureRecognizer>[];

  void _dispose() {
    for (final one in recognizers) {
      one.dispose();
    }
    recognizers = <TapGestureRecognizer>[];
  }

  @override
  void dispose() {
    _dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    _dispose();
    final (spans, taps) = linkedSpans(
      widget.ai.text,
      linkColor: colors.link,
      onOpen: widget.onOpenNote,
    );
    recognizers = taps;
    final long = widget.ai.text.length > overviewAiShortChars;
    return Container(
      margin: const EdgeInsets.fromLTRB(
        KansoSpacing.lg,
        KansoSpacing.lg,
        KansoSpacing.md,
        0,
      ),
      padding: const EdgeInsetsDirectional.only(start: KansoSpacing.sm),
      decoration: BoxDecoration(
        border: BorderDirectional(
          start: BorderSide(
            color: colors.accent.withValues(alpha: 0.45),
            width: 2,
          ),
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          Text.rich(
            TextSpan(children: spans),
            maxLines: widget.more || !long ? null : 6,
            overflow: TextOverflow.ellipsis,
            style: typography.body.copyWith(color: colors.text, height: 1.45),
          ),
          if (long && !widget.more)
            KansoPressable(
              onTap: widget.onMore,
              child: Padding(
                padding: const EdgeInsets.only(top: 4),
                child: Text(
                  l10n.overviewMore,
                  style: typography.label.copyWith(color: colors.link),
                ),
              ),
            ),
          if (widget.ai.warning case final warning?)
            Padding(
              padding: const EdgeInsets.only(top: 6),
              child: Text(
                warning,
                style: typography.meta.copyWith(color: colors.danger),
              ),
            ),
          Padding(
            padding: const EdgeInsets.only(top: KansoSpacing.xs),
            child: Wrap(
              spacing: KansoSpacing.sm,
              crossAxisAlignment: WrapCrossAlignment.center,
              children: <Widget>[
                KansoIcon(KansoIcons.sparkles, size: 12, color: colors.accent),
                Text(
                  l10n.overviewWrittenBy(widget.ai.timeLabel),
                  style: typography.meta.copyWith(color: colors.textMuted),
                ),
                KansoPressable(
                  onTap: widget.onRewrite,
                  child: Text(
                    l10n.overviewRewrite,
                    style: typography.meta.copyWith(
                      color: colors.textSecondary,
                    ),
                  ),
                ),
                KansoPressable(
                  onTap: widget.onHide,
                  child: Text(
                    l10n.overviewHide,
                    style: typography.meta.copyWith(
                      color: colors.textSecondary,
                    ),
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
