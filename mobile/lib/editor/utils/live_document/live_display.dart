// Kanso — live_display.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../note/views/widgets/preview/preview_due_chip.dart';
import '../../../note/views/widgets/preview/preview_style.dart';
import '../../../note/views/widgets/preview/preview_text.dart';
import 'live_layout_data.dart';
import 'live_rows.dart';

/// A stretch of a row's source and what is drawn for it.
@immutable
class LiveSegment {
  const LiveSegment({
    required this.source,
    required this.display,
    required this.kind,
    this.opens = false,
  });

  final TextRange source;
  final TextRange display;
  final LiveSegmentKind kind;

  /// A hidden mark that opens what comes after it (`**` before bold text,
  /// `[` before a link's text): a place next to it is the marked text's.
  final bool opens;
}

/// What is drawn for a stretch of source.
enum LiveSegmentKind {
  /// The source's own characters (a line break as a space or a break).
  text,

  /// Nothing: a hidden mark.
  hidden,

  /// One placeholder: a due chip or a formula.
  widget,
}

/// A row's text as drawn: the spans, and the way between units of the note
/// and of the drawn text.
@immutable
class LiveDisplay {
  const LiveDisplay({
    required this.span,
    required this.segments,
    required this.length,
    required this.base,
  });

  /// What is drawn, its style the row's.
  final TextSpan span;

  /// The row's source, start to end, cut where what is drawn changes.
  final List<LiveSegment> segments;

  /// How many units the drawn text has (a placeholder is one).
  final int length;

  /// The row's text style.
  final TextStyle base;

  bool get isEmpty => length == 0;

  /// Where [offset] of the note is in the drawn text: the end of what is
  /// drawn before a hidden mark, either end of a placeholder.
  int toDisplay(int offset) {
    for (final segment in segments) {
      if (offset > segment.source.end) {
        continue;
      }
      if (offset < segment.source.start) {
        return segment.display.start;
      }
      switch (segment.kind) {
        case LiveSegmentKind.text:
          return segment.display.start + (offset - segment.source.start);
        case LiveSegmentKind.hidden:
          return segment.display.start;
        case LiveSegmentKind.widget:
          final middle = (segment.source.start + segment.source.end) / 2;
          return offset <= middle ? segment.display.start : segment.display.end;
      }
    }
    return length;
  }

  /// Where [offset] of the drawn text is in the note. Between a mark and
  /// the text it marks, the text's side.
  int toSource(int offset) {
    for (var i = 0; i < segments.length; i++) {
      final segment = segments[i];
      if (segment.kind == LiveSegmentKind.hidden ||
          offset < segment.display.start ||
          offset > segment.display.end) {
        continue;
      }
      if (segment.kind == LiveSegmentKind.widget) {
        return offset <= segment.display.start
            ? segment.source.start
            : segment.source.end;
      }
      if (offset == segment.display.end) {
        // Before a mark that opens what follows: past it, into the text.
        var opens = false;
        var next = i + 1;
        while (next < segments.length &&
            segments[next].kind == LiveSegmentKind.hidden) {
          opens = opens || segments[next].opens;
          next++;
        }
        if (opens &&
            next < segments.length &&
            segments[next].kind == LiveSegmentKind.text &&
            segments[next].display.start == offset) {
          return segments[next].source.start;
        }
      }
      return segment.source.start + (offset - segment.display.start);
    }
    return segments.isEmpty ? 0 : segments.last.source.end;
  }
}

/// What is laid over a row's text: the marks of comments, other people's
/// selections, the text being composed.
@immutable
class LiveOverlays {
  const LiveOverlays({
    this.comments = const <TextRange>[],
    this.commentStyle = const TextStyle(),
    this.selections = const <(TextRange, Color)>[],
    this.composing = TextRange.empty,
  });

  final List<TextRange> comments;
  final TextStyle commentStyle;
  final List<(TextRange, Color)> selections;
  final TextRange composing;
}

final RegExp _due = RegExp(
  r'(?<![\w@])@\d{4}-\d{2}-\d{2}(?:[T ]\d{2}:\d{2})?(?![\w])',
);

final RegExp _tag = RegExp(
  r'(?<![\w#&/])#[\p{L}\p{N}_][\p{L}\p{N}_/-]*',
  unicode: true,
);

/// The text of [row] of [layout] as drawn in [style]: the marks hidden off
/// the caret, inline code, links, due chips and formulas as the preview
/// draws them.
LiveDisplay buildLiveDisplay({
  required LiveLayoutData layout,
  required LiveRow row,
  required PreviewStyle style,
  LiveOverlays overlays = const LiveOverlays(),
}) {
  final text = layout.text;
  final range = row.range;
  final tone = switch (row.tone) {
    LiveTone.body => style.text,
    LiveTone.quote => style.quote,
    LiveTone.done => style.taskDone,
  };
  final look = style.withText(tone);
  final base = row.headingLevel > 0
      ? look.heading(row.headingLevel)
      : look.body;
  bool inRow(TextRange r) => r.end > range.start && r.start < range.end;
  TextRange clip(TextRange r) => TextRange(
    start: r.start < range.start ? range.start : r.start,
    end: r.end > range.end ? range.end : r.end,
  );

  // What is hidden.
  final hidden = <TextRange>[
    for (final r in layout.conceal)
      if (inRow(r)) clip(r),
    for (final mark in layout.marks)
      if (inRow(mark.range)) clip(mark.range),
  ]..sort((a, b) => a.start.compareTo(b.start));
  bool isHidden(int at) {
    for (final r in hidden) {
      if (r.start > at) {
        return false;
      }
      if (at < r.end) {
        return true;
      }
    }
    return false;
  }

  // The preview's inlines of this block, for its due chips and formulas.
  final leaf = row.preview;
  final inlines = switch (leaf) {
    PreviewBlock_Paragraph(:final inlines) => inlines,
    PreviewBlock_Heading(:final inlines) => inlines,
    _ => const <PreviewInline>[],
  };
  final dues = <PreviewInline_Due>[];
  final maths = <PreviewInline_Math>[];
  void collect(List<PreviewInline> list) {
    for (final inline in list) {
      switch (inline) {
        case final PreviewInline_Due due:
          dues.add(due);
        case final PreviewInline_Math math:
          maths.add(math);
        case PreviewInline_Link(:final inlines):
          collect(inlines);
        default:
          break;
      }
    }
  }

  collect(inlines);

  // Placeholders: formulas off the caret, and due chips.
  final placeholders = <(TextRange, InlineSpan)>[];
  final formulas = <TextRange>[
    for (final s in layout.styles)
      if (s.kind == LiveStyleKind.inlineMath &&
          inRow(s.range) &&
          s.range.start > range.start &&
          isHidden(s.range.start - 1))
        s.range,
  ];
  if (formulas.length == maths.length) {
    for (var i = 0; i < formulas.length; i++) {
      // The formula with its dollar signs.
      final source = TextRange(
        start: formulas[i].start - 1,
        end: formulas[i].end + 1 <= range.end
            ? formulas[i].end + 1
            : formulas[i].end,
      );
      placeholders.add((
        source,
        WidgetSpan(
          alignment: PlaceholderAlignment.baseline,
          baseline: TextBaseline.alphabetic,
          child: PreviewInlineMath(maths[i].svg),
        ),
      ));
    }
  }
  if (!row.revealed && dues.isNotEmpty) {
    final found = _due
        .allMatches(text.substring(range.start, range.end))
        .map(
          (m) => TextRange(
            start: range.start + m.start,
            end: range.start + m.end,
          ),
        )
        .where((r) => !isHidden(r.start))
        .toList();
    if (found.length == dues.length) {
      for (var i = 0; i < found.length; i++) {
        final due = dues[i];
        placeholders.add((
          found[i],
          WidgetSpan(
            alignment: PlaceholderAlignment.middle,
            child: PreviewDueChip(
              label: due.label,
              title: due.title,
              tone: due.tone,
              size: base.fontSize ?? style.size,
            ),
          ),
        ));
      }
    }
  }
  placeholders.sort((a, b) => a.$1.start.compareTo(b.$1.start));

  // Where anything changes.
  final cuts = <int>{range.start, range.end};
  void cut(TextRange r) {
    if (inRow(r)) {
      final c = clip(r);
      cuts
        ..add(c.start)
        ..add(c.end);
    }
  }

  hidden.forEach(cut);
  for (final p in placeholders) {
    cut(p.$1);
  }
  for (final s in layout.styles) {
    cut(s.range);
  }
  for (final link in layout.links) {
    cut(link.text);
  }
  overlays.comments.forEach(cut);
  for (final s in overlays.selections) {
    cut(s.$1);
  }
  if (overlays.composing.isValid && !overlays.composing.isCollapsed) {
    cut(overlays.composing);
  }
  // Tags, which the page draws as links.
  final tags = <TextRange>[];
  for (final m in _tag.allMatches(text.substring(range.start, range.end))) {
    final r = TextRange(
      start: range.start + m.start,
      end: range.start + m.end,
    );
    tags.add(r);
    cut(r);
  }
  // Line breaks are a character of their own.
  for (var at = range.start; at < range.end; at++) {
    if (text.codeUnitAt(at) == 0x0A) {
      cuts
        ..add(at)
        ..add(at + 1);
    }
  }
  final points = cuts.where((c) => c >= range.start && c <= range.end).toList()
    ..sort();

  bool covered(List<TextRange> ranges, int at) {
    for (final r in ranges) {
      if (r.start <= at && at < r.end) {
        return true;
      }
    }
    return false;
  }

  bool styledAt(LiveStyleKind kind, int at) {
    for (final s in layout.styles) {
      if (s.range.start > at) {
        break;
      }
      if (s.kind == kind && at < s.range.end) {
        return true;
      }
    }
    return false;
  }

  LiveLinkData? linkAt(int at) {
    for (final link in layout.links) {
      if (link.text.start <= at && at < link.text.end) {
        return link;
      }
    }
    return null;
  }

  final segments = <LiveSegment>[];
  final spans = <InlineSpan>[];
  final buffer = StringBuffer();
  TextStyle? pending;
  var shown = 0;
  void flush() {
    if (buffer.isNotEmpty) {
      spans.add(TextSpan(text: buffer.toString(), style: pending));
      buffer.clear();
    }
  }

  void addText(String chunk, TextStyle style) {
    if (pending != style) {
      flush();
      pending = style;
    }
    buffer.write(chunk);
  }

  var p = 0;
  for (var i = 0; i + 1 < points.length; i++) {
    final from = points[i];
    final to = points[i + 1];
    if (from >= to) {
      continue;
    }
    // A placeholder.
    while (p < placeholders.length && placeholders[p].$1.end <= from) {
      p++;
    }
    if (p < placeholders.length &&
        placeholders[p].$1.start <= from &&
        from < placeholders[p].$1.end) {
      final holder = placeholders[p];
      if (from == holder.$1.start) {
        flush();
        pending = null;
        spans.add(holder.$2);
        final end = holder.$1.end > range.end ? range.end : holder.$1.end;
        segments.add(
          LiveSegment(
            source: TextRange(start: from, end: end),
            display: TextRange(start: shown, end: shown + 1),
            kind: LiveSegmentKind.widget,
          ),
        );
        shown += 1;
      }
      continue;
    }
    if (isHidden(from)) {
      segments.add(
        LiveSegment(
          source: TextRange(start: from, end: to),
          display: TextRange(start: shown, end: shown),
          kind: LiveSegmentKind.hidden,
          opens:
              layout.styles.any((s) => s.range.start == to) ||
              layout.links.any((link) => link.text.start == to),
        ),
      );
      continue;
    }
    var chunk = text.substring(from, to);
    if (chunk == '\n') {
      // A line whose every character is hidden takes its line break with
      // it; off the caret, a line break in a paragraph is a space, as on
      // the page, unless the line ends in a hard break.
      final nextEnd = text.indexOf('\n', to);
      final lineEnd = nextEnd < 0 || nextEnd > range.end ? range.end : nextEnd;
      var allHidden = lineEnd > to;
      for (var at = to; at < lineEnd && allHidden; at++) {
        allHidden = isHidden(at);
      }
      if (allHidden) {
        segments.add(
          LiveSegment(
            source: TextRange(start: from, end: to),
            display: TextRange(start: shown, end: shown),
            kind: LiveSegmentKind.hidden,
          ),
        );
        continue;
      }
      if (!row.revealed && row.kind != LiveRowKind.hidden) {
        final line = text.substring(
          text.lastIndexOf('\n', from - 1) + 1,
          from,
        );
        final hard = line.endsWith('  ') || line.endsWith(r'\');
        chunk = hard ? '\n' : ' ';
      }
    }
    // A wiki-link to no note is a link all the same: the page's link
    // makes the note.
    final code = styledAt(LiveStyleKind.code, from);
    final isLink = linkAt(from) != null || (!code && covered(tags, from));
    final mark = TextStyle(
      fontWeight: styledAt(LiveStyleKind.strong, from) ? FontWeight.w700 : null,
      fontStyle: styledAt(LiveStyleKind.emphasis, from)
          ? FontStyle.italic
          : null,
      decoration: styledAt(LiveStyleKind.strikethrough, from)
          ? TextDecoration.lineThrough
          : null,
    ).merge(code ? style.inlineCode : null);
    var drawn = isLink ? style.link.merge(mark) : mark;
    if (styledAt(LiveStyleKind.cardSeparator, from)) {
      drawn = drawn.merge(TextStyle(color: style.muted));
    }
    if (covered(overlays.comments, from)) {
      drawn = drawn.merge(overlays.commentStyle);
    }
    for (final (selected, color) in overlays.selections) {
      if (selected.start <= from && from < selected.end) {
        drawn = drawn.merge(TextStyle(backgroundColor: color));
      }
    }
    final composing = overlays.composing;
    if (composing.isValid && composing.start <= from && from < composing.end) {
      drawn = drawn.merge(
        TextStyle(
          decoration: TextDecoration.combine(<TextDecoration>[
            ?drawn.decoration,
            TextDecoration.underline,
          ]),
        ),
      );
    }
    addText(chunk, drawn);
    segments.add(
      LiveSegment(
        source: TextRange(start: from, end: to),
        display: TextRange(start: shown, end: shown + chunk.length),
        kind: LiveSegmentKind.text,
      ),
    );
    shown += chunk.length;
  }
  flush();
  if (segments.isEmpty) {
    segments.add(
      LiveSegment(
        source: TextRange(start: range.start, end: range.end),
        display: const TextRange(start: 0, end: 0),
        kind: LiveSegmentKind.hidden,
      ),
    );
  }
  return LiveDisplay(
    span: TextSpan(style: base, children: spans),
    segments: List<LiveSegment>.unmodifiable(segments),
    length: shown,
    base: base,
  );
}
