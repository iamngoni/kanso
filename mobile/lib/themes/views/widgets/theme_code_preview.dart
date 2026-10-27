// Kanso — theme_code_preview.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/parse_inline_markdown.dart';
import '../../models/data/theme_preview.dart';

/// Four numbered lines of Markdown in a theme's own colours: a heading, a
/// line with bold and italic, a line with code and a link, and a quote.
class ThemeCodePreview extends StatelessWidget {
  const ThemeCodePreview({required this.theme, super.key});

  final ThemePreview theme;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final base = context.typography.monoSmall.copyWith(color: theme.text);
    final number = base.copyWith(color: theme.marker);

    InlineSpan inline(String source) {
      return TextSpan(
        children: <InlineSpan>[
          for (final run in parseInlineMarkdown(source))
            TextSpan(
              text: run.text,
              style: switch (run.kind) {
                InlineKind.plain => null,
                InlineKind.bold => TextStyle(
                  color: theme.strong,
                  fontWeight: FontWeight.w700,
                ),
                InlineKind.italic => TextStyle(
                  color: theme.emphasis,
                  fontStyle: FontStyle.italic,
                ),
                InlineKind.code => TextStyle(color: theme.code),
                InlineKind.link => TextStyle(color: theme.link),
                InlineKind.url => TextStyle(color: theme.url),
                InlineKind.wiki => TextStyle(color: theme.link),
                InlineKind.marker => TextStyle(color: theme.marker),
              },
            ),
        ],
      );
    }

    final lines = <InlineSpan>[
      TextSpan(
        children: <InlineSpan>[
          TextSpan(
            text: '# ',
            style: TextStyle(color: theme.marker),
          ),
          TextSpan(
            text: l10n.themesSampleHeading,
            style: TextStyle(color: theme.heading, fontWeight: FontWeight.w700),
          ),
        ],
      ),
      inline(l10n.themesSampleLine),
      inline(l10n.themesSampleCode),
      TextSpan(
        children: <InlineSpan>[
          TextSpan(
            text: '> ',
            style: TextStyle(color: theme.marker),
          ),
          TextSpan(
            text: l10n.themesSampleQuote,
            style: TextStyle(color: theme.quote, fontStyle: FontStyle.italic),
          ),
        ],
      ),
    ];
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.all(KansoSpacing.sm),
      decoration: BoxDecoration(
        color: theme.editor,
        borderRadius: const BorderRadius.vertical(
          top: Radius.circular(KansoRadii.medium),
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: <Widget>[
          for (var i = 0; i < lines.length; i++)
            Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: <Widget>[
                SizedBox(
                  width: KansoSpacing.md,
                  child: Text('${i + 1}', style: number),
                ),
                const SizedBox(width: KansoSpacing.xs),
                Expanded(
                  child: Text.rich(
                    TextSpan(style: base, children: <InlineSpan>[lines[i]]),
                    maxLines: 1,
                    overflow: TextOverflow.clip,
                    softWrap: false,
                  ),
                ),
              ],
            ),
        ],
      ),
    );
  }
}
