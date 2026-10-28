// Kanso — answer_text.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/utils/context_extensions.dart';
import '../../models/data/answer_span.dart';
import '../../utils/parse_answer_inline.dart';
import 'citation_chip.dart';

/// A run of an answer's inline Markdown: bold, italic and code in the text's
/// own style, a citation as a [CitationChip], a web link as underlined text.
class AnswerInline extends StatelessWidget {
  const AnswerInline({
    required this.source,
    required this.style,
    required this.onOpenNote,
    required this.onOpenLink,
    required this.usedNoteIds,
    super.key,
  });

  final String source;
  final TextStyle style;
  final ValueChanged<String> onOpenNote;
  final ValueChanged<String> onOpenLink;
  final Set<String> usedNoteIds;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final mono = context.typography.monoSmall;
    final spans = parseAnswerInline(source, usedNoteIds: usedNoteIds);
    return Text.rich(
      TextSpan(
        style: style,
        children: <InlineSpan>[
          for (final span in spans)
            switch (span) {
              AnswerText(
                :final text,
                :final bold,
                :final italic,
                :final code,
              ) =>
                TextSpan(
                  text: text,
                  style: code
                      ? mono.copyWith(backgroundColor: colors.fill)
                      : TextStyle(
                          fontWeight: bold ? FontWeight.w600 : null,
                          fontStyle: italic ? FontStyle.italic : null,
                        ),
                ),
              AnswerNoteLink(:final text, :final noteId) => WidgetSpan(
                alignment: PlaceholderAlignment.baseline,
                baseline: TextBaseline.alphabetic,
                child: CitationChip(
                  label: text,
                  onTap: () => onOpenNote(noteId),
                ),
              ),
              AnswerWebLink(:final text, :final url) => WidgetSpan(
                alignment: PlaceholderAlignment.baseline,
                baseline: TextBaseline.alphabetic,
                child: CitationChip(
                  label: text,
                  web: true,
                  onTap: () => onOpenLink(url),
                ),
              ),
            },
        ],
      ),
    );
  }
}
