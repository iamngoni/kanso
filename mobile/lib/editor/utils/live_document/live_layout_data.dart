// Kanso — live_layout_data.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

/// What a block of a note is, as Live lays it out.
enum LiveBlockKind {
  paragraph,
  heading,
  rule,
  code,
  table,
  math,
  html,
  frontMatter,
  image,
  definition,
  blank;

  /// Whether a block of this kind is drawn by the preview's renderer while
  /// the caret is off it.
  bool get isWidget => switch (this) {
    code || table || math || html || frontMatter || image => true,
    _ => false,
  };
}

/// What a hidden block mark is drawn as.
enum LiveMarkKind {
  bullet,
  number,
  task,
  hidden,
  quote,
  indent,
  rule,
  underline,
}

/// How a span of the text is drawn.
enum LiveStyleKind {
  strong,
  emphasis,
  strikethrough,
  code,
  link,
  heading,
  inlineMath,
  quote,

  /// The `::` of a flashcard line, drawn dim.
  cardSeparator,
  taskDone,
}

/// A block of the note: its whole lines, in units of the text.
@immutable
class LiveBlockData {
  const LiveBlockData({
    required this.kind,
    required this.lines,
    this.level = 0,
    this.listDepth = 0,
    this.quoteDepth = 0,
    this.itemStart = false,
    this.top,
    this.revealed = false,
  });

  final LiveBlockKind kind;

  /// A heading's level; 0 for anything else.
  final int level;

  /// From the start of its first line to the end of its last, without the
  /// line break after it.
  final TextRange lines;

  /// How many list items it is in.
  final int listDepth;

  /// How many quotes it is in.
  final int quoteDepth;

  /// Whether it is the first block of a list item.
  final bool itemStart;

  /// Which of the note's top-level blocks it is in; null between them.
  final int? top;

  /// Whether the caret or the selection is on one of its lines.
  final bool revealed;

  LiveBlockData _moved(TextRange lines) => LiveBlockData(
    kind: kind,
    lines: lines,
    level: level,
    listDepth: listDepth,
    quoteDepth: quoteDepth,
    itemStart: itemStart,
    top: top,
    revealed: revealed,
  );
}

/// A block mark hidden, and what is drawn for it.
@immutable
class LiveMarkData {
  const LiveMarkData({
    required this.range,
    required this.kind,
    this.depth = 0,
    this.text = '',
    this.checked = false,
  });

  final TextRange range;
  final LiveMarkKind kind;
  final int depth;

  /// A bullet's glyph or a number, as the preview draws them.
  final String text;

  /// Whether a task's box is ticked.
  final bool checked;
}

/// A span of the text drawn in a style.
@immutable
class LiveStyleData {
  const LiveStyleData({required this.range, required this.kind});

  final TextRange range;
  final LiveStyleKind kind;
}

/// A link: all of it, the text drawn for it, and whether it is a wiki-link
/// to no note.
@immutable
class LiveLinkData {
  const LiveLinkData({
    required this.range,
    required this.text,
    this.missing = false,
  });

  final TextRange range;
  final TextRange text;
  final bool missing;
}

/// A block drawn by the preview's renderer: its whole lines.
@immutable
class LiveWidgetData {
  const LiveWidgetData({required this.kind, required this.lines});

  final LiveBlockKind kind;
  final TextRange lines;
}

/// What Live draws for a note's [text]: the engine's layout, in units of
/// the text, as of the version it was worked out for.
///
/// A layout that comes back for a text a few keys old is moved along the
/// change ([shiftedTo]) until the next one comes, so what is hidden never
/// outlives the text it was hidden in.
@immutable
class LiveLayoutData {
  const LiveLayoutData({
    required this.text,
    required this.version,
    this.conceal = const <TextRange>[],
    this.blocks = const <LiveBlockData>[],
    this.marks = const <LiveMarkData>[],
    this.styles = const <LiveStyleData>[],
    this.links = const <LiveLinkData>[],
    this.widgets = const <LiveWidgetData>[],
  });

  /// The text this is the layout of.
  final String text;
  final int version;
  final List<TextRange> conceal;
  final List<LiveBlockData> blocks;
  final List<LiveMarkData> marks;
  final List<LiveStyleData> styles;
  final List<LiveLinkData> links;
  final List<LiveWidgetData> widgets;

  /// Where each line of [text] starts.
  static List<int> lineStarts(String text) {
    final starts = <int>[0];
    for (var at = 0; at < text.length; at++) {
      if (text.codeUnitAt(at) == 0x0A) {
        starts.add(at + 1);
      }
    }
    return starts;
  }

  /// Lines [first] to [end] (exclusive) of a text with lines starting at
  /// [starts] and [length] units long, as a range of whole lines without
  /// the last line break.
  static TextRange linesRange(
    List<int> starts,
    int length,
    int first,
    int end,
  ) {
    final from = first.clamp(0, starts.length - 1);
    final last = (end - 1).clamp(from, starts.length - 1);
    final to = last + 1 < starts.length ? starts[last + 1] - 1 : length;
    return TextRange(
      start: starts[from],
      end: to < starts[from] ? starts[from] : to,
    );
  }

  /// The plain layout of [text] with nothing hidden: one block per run of
  /// lines, before the core has said anything.
  factory LiveLayoutData.plain(String text, {int version = 0}) {
    final starts = lineStarts(text);
    return LiveLayoutData(
      text: text,
      version: version,
      blocks: <LiveBlockData>[
        for (var line = 0; line < starts.length; line++)
          LiveBlockData(
            kind: LiveBlockKind.paragraph,
            lines: linesRange(starts, text.length, line, line + 1),
            revealed: true,
          ),
      ],
    );
  }

  /// This layout moved along the change from [text] to [now]: the units
  /// before the change stay, those after it move by its length, and a
  /// block the change is in, or ends at, grows to take in what was typed.
  LiveLayoutData shiftedTo(String now, {int? version}) {
    if (identical(now, text) || now == text) {
      return this;
    }
    final old = text;
    var start = 0;
    final shorter = old.length < now.length ? old.length : now.length;
    while (start < shorter && old.codeUnitAt(start) == now.codeUnitAt(start)) {
      start++;
    }
    var oldEnd = old.length;
    var newEnd = now.length;
    while (oldEnd > start &&
        newEnd > start &&
        old.codeUnitAt(oldEnd - 1) == now.codeUnitAt(newEnd - 1)) {
      oldEnd--;
      newEnd--;
    }
    final delta = newEnd - oldEnd;
    // A start inside the change goes to its start; an end inside it goes
    // to its start too, unless the range grows (a block), which takes in
    // what was typed at its end.
    int startOf(int at) {
      if (at <= start) {
        return at;
      }
      if (at >= oldEnd) {
        return at + delta;
      }
      return start;
    }

    int endOf(int at, {required bool grow}) {
      if (grow) {
        if (at < start) {
          return at;
        }
        if (at >= oldEnd) {
          return at + delta;
        }
        return newEnd;
      }
      if (at <= start) {
        return at;
      }
      if (at >= oldEnd) {
        return at + delta;
      }
      return start;
    }

    TextRange fixed(TextRange range) {
      final from = startOf(range.start);
      final to = endOf(range.end, grow: false);
      return TextRange(start: from, end: to < from ? from : to);
    }

    TextRange grown(TextRange range) {
      final from = startOf(range.start);
      final to = endOf(range.end, grow: true);
      return TextRange(start: from, end: to < from ? from : to);
    }

    bool keep(TextRange range) => !range.isCollapsed;

    return LiveLayoutData(
      text: now,
      version: version ?? this.version,
      conceal: <TextRange>[
        for (final range in conceal)
          if (keep(fixed(range))) fixed(range),
      ],
      blocks: <LiveBlockData>[
        for (final block in blocks) block._moved(grown(block.lines)),
      ],
      marks: <LiveMarkData>[
        for (final mark in marks)
          if (keep(fixed(mark.range)))
            LiveMarkData(
              range: fixed(mark.range),
              kind: mark.kind,
              depth: mark.depth,
              text: mark.text,
              checked: mark.checked,
            ),
      ],
      styles: <LiveStyleData>[
        for (final style in styles)
          if (keep(fixed(style.range)))
            LiveStyleData(range: fixed(style.range), kind: style.kind),
      ],
      links: <LiveLinkData>[
        for (final link in links)
          LiveLinkData(
            range: fixed(link.range),
            text: fixed(link.text),
            missing: link.missing,
          ),
      ],
      widgets: <LiveWidgetData>[
        for (final widget in widgets)
          LiveWidgetData(kind: widget.kind, lines: grown(widget.lines)),
      ],
    );
  }
}
