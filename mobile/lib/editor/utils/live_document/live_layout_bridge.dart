// Kanso — live_layout_bridge.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/services.dart';

import '../../../core/bridge/api/types.dart';
import 'live_layout_data.dart';
import 'live_rows.dart';

/// [document], the core's Live layout of [text], in units of the text.
LiveLayoutData liveLayoutOf(LiveDocument document, String text) {
  final starts = LiveLayoutData.lineStarts(text);
  TextRange lines(int first, int end) =>
      LiveLayoutData.linesRange(starts, text.length, first, end);
  TextRange range(Range16 r) {
    final start = r.start.clamp(0, text.length);
    final end = r.end.clamp(0, text.length);
    return start <= end
        ? TextRange(start: start, end: end)
        : TextRange(start: end, end: start);
  }

  return LiveLayoutData(
    text: text,
    version: document.version,
    conceal: <TextRange>[for (final r in document.conceal) range(r)],
    blocks: <LiveBlockData>[
      for (final block in document.blocks)
        LiveBlockData(
          kind: _kind(block.kind),
          level: block.level,
          lines: lines(block.firstLine, block.endLine),
          listDepth: block.listDepth,
          quoteDepth: block.quoteDepth,
          itemStart: block.itemStart,
          top: block.top,
          revealed: block.revealed,
        ),
    ],
    marks: <LiveMarkData>[
      for (final mark in document.markers)
        LiveMarkData(
          range: range(mark.range),
          kind: switch (mark.kind) {
            LiveDocMarkerKind.bullet => LiveMarkKind.bullet,
            LiveDocMarkerKind.number => LiveMarkKind.number,
            LiveDocMarkerKind.task => LiveMarkKind.task,
            LiveDocMarkerKind.hidden => LiveMarkKind.hidden,
            LiveDocMarkerKind.quote => LiveMarkKind.quote,
            LiveDocMarkerKind.indent => LiveMarkKind.indent,
            LiveDocMarkerKind.rule => LiveMarkKind.rule,
            LiveDocMarkerKind.underline => LiveMarkKind.underline,
          },
          depth: mark.depth,
          text: mark.text,
          checked: mark.checked,
        ),
    ],
    styles: <LiveStyleData>[
      for (final style in document.styles)
        LiveStyleData(
          range: range(style.range),
          kind: switch (style.kind) {
            LiveDocStyleKind.strong => LiveStyleKind.strong,
            LiveDocStyleKind.emphasis => LiveStyleKind.emphasis,
            LiveDocStyleKind.strikethrough => LiveStyleKind.strikethrough,
            LiveDocStyleKind.code => LiveStyleKind.code,
            LiveDocStyleKind.link => LiveStyleKind.link,
            LiveDocStyleKind.heading => LiveStyleKind.heading,
            LiveDocStyleKind.inlineMath => LiveStyleKind.inlineMath,
            LiveDocStyleKind.quote => LiveStyleKind.quote,
            LiveDocStyleKind.cardSeparator => LiveStyleKind.cardSeparator,
            LiveDocStyleKind.taskDone => LiveStyleKind.taskDone,
          },
        ),
    ],
    links: <LiveLinkData>[
      for (final link in document.links)
        LiveLinkData(
          range: range(link.range),
          text: range(link.text),
          missing: link.missing,
        ),
    ],
    widgets: <LiveWidgetData>[
      for (final widget in document.widgets)
        LiveWidgetData(
          kind: _kind(widget.kind),
          lines: lines(widget.firstLine, widget.endLine),
        ),
    ],
  );
}

/// [blocks], the preview's top-level blocks of [text], with the Markdown
/// each was drawn from; the footnotes the preview adds have none and are
/// left out.
List<LivePreviewBlock> livePreviewBlocksOf(
  List<PreviewSourcedBlock> blocks,
  String text,
) {
  final starts = LiveLayoutData.lineStarts(text);
  return <LivePreviewBlock>[
    for (final sourced in blocks)
      if (sourced.firstLine case final first?)
        (
          source: LiveLayoutData.linesRange(
            starts,
            text.length,
            first,
            sourced.endLine ?? first + 1,
          ).textInside(text),
          block: sourced.block,
        ),
  ];
}

LiveBlockKind _kind(LiveDocBlockKind kind) => switch (kind) {
  LiveDocBlockKind.paragraph => LiveBlockKind.paragraph,
  LiveDocBlockKind.heading => LiveBlockKind.heading,
  LiveDocBlockKind.rule => LiveBlockKind.rule,
  LiveDocBlockKind.code => LiveBlockKind.code,
  LiveDocBlockKind.table => LiveBlockKind.table,
  LiveDocBlockKind.math => LiveBlockKind.math,
  LiveDocBlockKind.html => LiveBlockKind.html,
  LiveDocBlockKind.frontMatter => LiveBlockKind.frontMatter,
  LiveDocBlockKind.image => LiveBlockKind.image,
  LiveDocBlockKind.definition => LiveBlockKind.definition,
  LiveDocBlockKind.blank => LiveBlockKind.blank,
};
