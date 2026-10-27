// Kanso — shift_runs.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/painting.dart' show TextRange;

import '../../core/bridge/api/types.dart';

/// [runs] moved to fit a text in which the units [start] to [end] were
/// replaced by [inserted] units, until the Rust core sends the colours of
/// the new text.
///
/// - A run before the change stays; a run after it moves by the change in
///   length.
/// - A run the change falls inside grows or shrinks with it, and so does the
///   run that ends where text is typed (typing on at the end of a word keeps
///   the word's colour until the fresh colours arrive).
/// - A run the change covers only in part is cut to what is left, with the
///   new text in it when it started before the change (so the new text is
///   in one run only); one it covers entirely goes.
///
/// The runs stay in order and never overlap.
List<EditorRun> shiftRuns(
  List<EditorRun> runs, {
  required int start,
  required int end,
  required int inserted,
}) {
  final delta = inserted - (end - start);
  final shifted = <EditorRun>[];
  for (final run in runs) {
    final from = run.range.start;
    final to = run.range.end;
    if (to <= start) {
      // Before the change; one that ends where text is typed grows.
      final grows = to == start && start == end && inserted > 0 && from < to;
      shifted.add(grows ? _with(run, from, to + inserted) : run);
    } else if (from >= end) {
      shifted.add(delta == 0 ? run : _with(run, from + delta, to + delta));
    } else if (from >= start && to <= end) {
      // Covered entirely by the change.
      continue;
    } else {
      final newFrom = from < start ? from : start + inserted;
      final newTo = to > end ? to + delta : start + inserted;
      if (newTo > newFrom) {
        shifted.add(_with(run, newFrom, newTo));
      }
    }
  }
  return shifted;
}

EditorRun _with(EditorRun run, int start, int end) {
  return EditorRun(
    range: Range16(start: start, end: end),
    color: run.color,
    italic: run.italic,
    weight: run.weight,
    quiet: run.quiet,
  );
}

/// [ranges] moved to fit a text in which the units [start] to [end] were
/// replaced by [inserted] units: a range before the change stays, one after
/// it moves, and one the change touches goes (what is marked in it is
/// checked again).
List<TextRange> shiftRanges(
  List<TextRange> ranges, {
  required int start,
  required int end,
  required int inserted,
}) {
  final delta = inserted - (end - start);
  return <TextRange>[
    for (final range in ranges)
      if (range.end <= start)
        range
      else if (range.start >= end)
        TextRange(start: range.start + delta, end: range.end + delta),
  ];
}
