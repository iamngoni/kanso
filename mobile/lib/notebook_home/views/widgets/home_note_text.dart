// Kanso — home_note_text.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/parse_inline_markdown.dart';

/// A Home note's text as preview reads it (issue #201): headings, list
/// items and task boxes laid out, and bold, italic, code and links styled
/// with their marks taken away. Nothing here loads a picture or opens a
/// link; the note's own page does that.
class HomeNoteText extends StatelessWidget {
  const HomeNoteText(this.source, {super.key});

  final String source;

  @override
  Widget build(BuildContext context) {
    final t = context.typography;
    final lines = source.split('\n');
    final children = <Widget>[];
    var blank = false;
    for (final raw in lines) {
      final line = raw.trimRight();
      if (line.trim().isEmpty) {
        blank = children.isNotEmpty;
        continue;
      }
      if (blank) {
        children.add(const SizedBox(height: KansoSpacing.xs));
        blank = false;
      }
      final heading = _heading.firstMatch(line);
      if (heading != null) {
        final style = heading.group(1)!.length == 1
            ? t.previewHeading1
            : t.previewHeading2;
        children.add(_rich(context, heading.group(2)!, style));
        continue;
      }
      final item = _item.firstMatch(line);
      if (item != null) {
        final box = item.group(2);
        final lead = switch (box?.toLowerCase()) {
          null => '•',
          'x' => '☑',
          _ => '☐',
        };
        final indent = item.group(1)!.length ~/ 2 * KansoSpacing.md;
        children.add(
          Padding(
            padding: EdgeInsetsDirectional.only(start: indent),
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: <Widget>[
                SizedBox(
                  width: KansoSpacing.lg,
                  child: Text(lead, style: t.previewBody),
                ),
                Expanded(
                  child: _rich(context, item.group(3)!, t.previewBody),
                ),
              ],
            ),
          ),
        );
        continue;
      }
      children.add(_rich(context, line.trimLeft(), t.previewBody));
    }
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: children,
    );
  }

  Widget _rich(BuildContext context, String text, TextStyle base) {
    final colors = context.colors;
    final spans = <InlineSpan>[];
    for (final run in parseInlineMarkdown(text)) {
      switch (run.kind) {
        case InlineKind.marker:
        case InlineKind.url:
          break;
        case InlineKind.plain:
          spans.add(TextSpan(text: run.text));
        case InlineKind.bold:
          spans.add(
            TextSpan(
              text: run.text,
              style: const TextStyle(fontWeight: FontWeight.w600),
            ),
          );
        case InlineKind.italic:
          spans.add(
            TextSpan(
              text: run.text,
              style: const TextStyle(fontStyle: FontStyle.italic),
            ),
          );
        case InlineKind.code:
          spans.add(
            TextSpan(
              text: run.text,
              style: context.typography.mono.copyWith(
                fontSize: base.fontSize,
              ),
            ),
          );
        case InlineKind.link:
        case InlineKind.wiki:
          spans.add(
            TextSpan(
              text: run.text,
              style: TextStyle(color: colors.accent),
            ),
          );
      }
    }
    return Text.rich(TextSpan(style: base, children: spans));
  }
}

final _heading = RegExp(r'^(#{1,6})\s+(.*)$');
final _item = RegExp(r'^(\s*)(?:[-*+]|\d+[.)])\s+(?:\[([ xX])\]\s+)?(.*)$');
