// Kanso — answer_block_view.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_divider.dart';
import '../../models/data/answer_block.dart';
import 'answer_inline.dart';

/// One block of an answer.
class AnswerBlockView extends StatelessWidget {
  const AnswerBlockView({
    required this.block,
    required this.onOpenNote,
    required this.onOpenLink,
    required this.usedNoteIds,
    super.key,
  });

  final AnswerBlock block;
  final ValueChanged<String> onOpenNote;
  final ValueChanged<String> onOpenLink;
  final Set<String> usedNoteIds;

  /// How far each level of a nested list is indented.
  static const double listIndent = 16;

  /// How wide the bullet or number of a list item is.
  static const double markerWidth = 22;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    AnswerInline text(String source, TextStyle style) => AnswerInline(
      source: source,
      style: style,
      onOpenNote: onOpenNote,
      onOpenLink: onOpenLink,
      usedNoteIds: usedNoteIds,
    );
    return switch (block) {
      AnswerHeading(:final level, :final text) => AnswerInline(
        source: text,
        style: switch (level) {
          1 => typography.heading1,
          2 => typography.heading2,
          _ => typography.listTitle,
        },
        onOpenNote: onOpenNote,
        onOpenLink: onOpenLink,
        usedNoteIds: usedNoteIds,
      ),
      AnswerParagraph(text: final source) => text(source, typography.body),
      AnswerList(:final items) => Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          for (final item in items)
            Padding(
              padding: EdgeInsetsDirectional.only(
                start: item.depth * listIndent,
                bottom: KansoSpacing.xxs,
              ),
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: <Widget>[
                  SizedBox(
                    width: markerWidth,
                    child: Text(
                      item.marker,
                      style: typography.body.copyWith(
                        color: colors.textSecondary,
                      ),
                    ),
                  ),
                  Expanded(child: text(item.text, typography.body)),
                ],
              ),
            ),
        ],
      ),
      AnswerCode(text: final code) => Container(
        padding: const EdgeInsets.all(KansoSpacing.sm),
        decoration: BoxDecoration(
          color: colors.fill,
          borderRadius: KansoRadii.mediumBorder,
        ),
        child: Text(code, style: typography.monoSmall),
      ),
      AnswerQuote(text: final source) => Container(
        padding: const EdgeInsetsDirectional.only(start: KansoSpacing.sm),
        decoration: BoxDecoration(
          border: BorderDirectional(
            start: BorderSide(color: colors.accent, width: 3),
          ),
        ),
        child: text(
          source,
          typography.body.copyWith(color: colors.textSecondary),
        ),
      ),
      AnswerRule() => const KansoDivider(),
    };
  }
}
