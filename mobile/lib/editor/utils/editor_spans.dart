// Kanso — editor_spans.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/painting.dart';

import '../../core/bridge/api/types.dart';
import '../../core/utils/run_style.dart';
import 'diff_values.dart';

/// The text of the editor as spans: each styled run in the colour, slant and
/// weight the theme gives it, the [composing] range underlined, and the
/// [misspellings] marked with [misspelledStyle], except where a run is
/// [EditorRun.quiet] (code, addresses, front matter).
///
/// The spans are the text cut at every place the style changes; joined, they
/// are [text] exactly. No span starts or ends between the two halves of a
/// surrogate pair, whatever the ranges say. [runs] must be in order and not
/// overlap; a run or range outside the text is cut to it.
///
/// A composing range is underlined with a plain line, and wins over a
/// misspelling where they meet.
TextSpan editorSpans({
  required String text,
  required List<EditorRun> runs,
  required TextStyle base,
  TextRange composing = TextRange.empty,
  List<TextRange> misspellings = const <TextRange>[],
  TextStyle composingStyle = const TextStyle(
    decoration: TextDecoration.underline,
  ),
  TextStyle misspelledStyle = const TextStyle(
    decoration: TextDecoration.underline,
    decorationStyle: TextDecorationStyle.wavy,
  ),
}) {
  final length = text.length;
  int snap(int at) {
    final clamped = at.clamp(0, length);
    return splitsPair(text, clamped) ? clamped - 1 : clamped;
  }

  // Where the style may change.
  final stops = <int>{0, length};
  for (final run in runs) {
    stops
      ..add(snap(run.range.start))
      ..add(snap(run.range.end));
  }
  final composed = composing.isValid && !composing.isCollapsed
      ? TextRange(start: snap(composing.start), end: snap(composing.end))
      : TextRange.empty;
  if (!composed.isCollapsed) {
    stops
      ..add(composed.start)
      ..add(composed.end);
  }
  final marks = <TextRange>[
    for (final range in misspellings)
      if (range.isValid && !range.isCollapsed)
        TextRange(start: snap(range.start), end: snap(range.end)),
  ];
  for (final mark in marks) {
    stops
      ..add(mark.start)
      ..add(mark.end);
  }
  final ordered = stops.toList()..sort();

  if (ordered.length <= 2 && runs.isEmpty && composed.isCollapsed) {
    return TextSpan(text: text, style: base);
  }

  // One sweep over the runs and the marks, which are in order.
  final spans = <InlineSpan>[];
  var run = 0;
  var mark = 0;
  final sortedMarks = marks..sort((a, b) => a.start.compareTo(b.start));
  for (var i = 0; i + 1 < ordered.length; i++) {
    final from = ordered[i];
    final to = ordered[i + 1];
    if (to <= from) {
      continue;
    }
    while (run < runs.length && snap(runs[run].range.end) <= from) {
      run++;
    }
    final here = run < runs.length && snap(runs[run].range.start) <= from
        ? runs[run]
        : null;
    // Marks can overlap each other: look at all that reach this span.
    while (mark < sortedMarks.length && sortedMarks[mark].end <= from) {
      mark++;
    }
    var misspelled = false;
    for (var m = mark; m < sortedMarks.length; m++) {
      if (sortedMarks[m].start > from) {
        break;
      }
      if (sortedMarks[m].end > from) {
        misspelled = true;
        break;
      }
    }
    final inComposing =
        !composed.isCollapsed && composed.start <= from && composed.end >= to;

    TextStyle? style = here == null
        ? null
        : runStyle(color: here.color, italic: here.italic, weight: here.weight);
    if (inComposing) {
      style = style?.merge(composingStyle) ?? composingStyle;
    } else if (misspelled && !(here?.quiet ?? false)) {
      style = style?.merge(misspelledStyle) ?? misspelledStyle;
    }
    spans.add(TextSpan(text: text.substring(from, to), style: style));
  }
  return TextSpan(style: base, children: spans);
}
