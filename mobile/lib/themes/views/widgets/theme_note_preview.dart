// Kanso — theme_note_preview.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/palette_from_tokens.dart';
import '../../models/data/theme_preview.dart';

/// A rendered note in a preview theme's colours on its own editor surface:
/// a heading, a line with bold, a link and inline code, a quote and a code
/// block. The picture the Preview cards show.
class ThemeNotePreview extends StatelessWidget {
  const ThemeNotePreview({required this.theme, super.key});

  final ThemePreview theme;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final typography = context.typography;
    final colors = theme.palette.preview;
    final syntax = theme.palette.syntax;
    final text = opaqueColor(colors.text);
    final codeText = opaqueColor(colors.codeText ?? colors.text);
    final body = typography.secondary.copyWith(color: text, height: 1.3);
    final code = typography.monoSmall.copyWith(color: codeText);
    final inlineCodeText = colors.inlineCodeText;
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
            Text(
              l10n.themesSampleHeading,
              maxLines: 1,
              overflow: TextOverflow.clip,
              style: typography.heading2.copyWith(
                color: opaqueColor(colors.heading ?? colors.text),
                fontWeight: FontWeight.w700,
              ),
            ),
            const SizedBox(height: KansoSpacing.xxs),
            Text.rich(
              TextSpan(
                style: body,
                children: <InlineSpan>[
                  TextSpan(text: '${l10n.themesThumbSome} '),
                  TextSpan(
                    text: '${l10n.themesThumbBoldComma} ',
                    style: const TextStyle(fontWeight: FontWeight.w700),
                  ),
                  TextSpan(
                    text: '${l10n.themesThumbLinkComma} ',
                    style: TextStyle(color: opaqueColor(colors.accent)),
                  ),
                  TextSpan(
                    text: l10n.themesThumbInlineCode,
                    style: code.copyWith(
                      color: inlineCodeText == null
                          ? text
                          : opaqueColor(inlineCodeText),
                      backgroundColor: opaqueColor(
                        colors.inlineCodeBackground,
                      ),
                    ),
                  ),
                ],
              ),
              maxLines: 1,
              overflow: TextOverflow.clip,
              softWrap: false,
            ),
            const SizedBox(height: KansoSpacing.xs),
            IntrinsicHeight(
              child: Row(
                children: <Widget>[
                  Container(width: 2, color: opaqueColor(colors.border)),
                  const SizedBox(width: KansoSpacing.xs),
                  Text(
                    l10n.themesSampleQuote,
                    maxLines: 1,
                    style: body.copyWith(color: opaqueColor(colors.quote)),
                  ),
                ],
              ),
            ),
            const SizedBox(height: KansoSpacing.xs),
            Container(
              width: double.infinity,
              padding: const EdgeInsets.symmetric(
                horizontal: KansoSpacing.xs,
                vertical: KansoSpacing.xxs,
              ),
              decoration: BoxDecoration(
                color: opaqueColor(colors.surface),
                borderRadius: BorderRadius.circular(KansoRadii.small),
              ),
              child: Text.rich(
                TextSpan(
                  style: code,
                  children: <InlineSpan>[
                    TextSpan(
                      text: 'const ',
                      style: TextStyle(color: syntax.keyword),
                    ),
                    TextSpan(
                      text: 'x',
                      style: TextStyle(color: syntax.variable),
                    ),
                    TextSpan(
                      text: ' = ',
                      style: TextStyle(color: syntax.operator),
                    ),
                    TextSpan(
                      text: '1',
                      style: TextStyle(color: syntax.number),
                    ),
                  ],
                ),
                maxLines: 1,
                overflow: TextOverflow.clip,
                softWrap: false,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
