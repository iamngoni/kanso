// Kanso — theme_syntax_preview.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../models/data/theme_preview.dart';

/// A few lines of Markdown and code set in a syntax theme's colours on its
/// own editor: a heading, bold and italic, code and a link, a task, a quote
/// and a line of code. The picture the Editor syntax cards show.
class ThemeSyntaxPreview extends StatelessWidget {
  const ThemeSyntaxPreview({required this.theme, super.key});

  final ThemePreview theme;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final syntax = theme.palette.syntax;
    final styles = theme.palette.syntaxStyles;
    final base = context.typography.monoSmall.copyWith(color: syntax.text);

    // [words] in the colour of [token], with the theme's italic or weight
    // for it.
    TextSpan span(String words, String token, Color color) {
      final style = styles[token];
      return TextSpan(
        text: words,
        style: TextStyle(
          color: color,
          fontStyle: (style?.italic ?? false) ? FontStyle.italic : null,
          fontWeight: style?.weight == null
              ? null
              : FontWeight.values[(style!.weight! ~/ 100 - 1).clamp(0, 8)],
        ),
      );
    }

    final lines = <List<TextSpan>>[
      <TextSpan>[
        span('# ', 'block_marker', syntax.blockMarker),
        span(l10n.themesSampleHeading, 'heading', syntax.heading),
      ],
      <TextSpan>[
        span('**', 'marker', syntax.marker),
        span(l10n.themesThumbBold, 'strong', theme.strong),
        span('**', 'marker', syntax.marker),
        span(l10n.themesThumbAnd, 'text', syntax.text),
        span('_', 'marker', syntax.marker),
        span(l10n.themesThumbItalic, 'emphasis', theme.emphasis),
        span('_', 'marker', syntax.marker),
      ],
      <TextSpan>[
        span('`${l10n.themesThumbInlineCode}`', 'code', syntax.code),
        span(', ', 'text', syntax.text),
        span('[', 'marker', syntax.marker),
        span(l10n.themesThumbLink, 'link', syntax.link),
        span('](', 'marker', syntax.marker),
        span('url', 'url', syntax.url),
        span(')', 'marker', syntax.marker),
      ],
      <TextSpan>[
        span('- ', 'list_marker', syntax.listMarker),
        span('[x] ', 'task', syntax.task),
        span(l10n.themesThumbDone, 'text', syntax.text),
      ],
      <TextSpan>[
        span('> ', 'block_marker', syntax.blockMarker),
        span(l10n.themesSampleQuote, 'quote', theme.quote),
      ],
      <TextSpan>[
        span('let ', 'keyword', syntax.keyword),
        span('n', 'variable', syntax.variable),
        span(' = ', 'operator', syntax.operator),
        span('"hi"', 'string', syntax.string),
        span('; ', 'punctuation', syntax.punctuation),
        span('// 42', 'comment', syntax.comment),
      ],
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
      child: ExcludeSemantics(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: <Widget>[
            for (final line in lines)
              Text.rich(
                TextSpan(style: base, children: line),
                maxLines: 1,
                overflow: TextOverflow.clip,
                softWrap: false,
              ),
          ],
        ),
      ),
    );
  }
}
