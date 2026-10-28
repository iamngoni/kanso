// Kanso — shift_marks.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/painting.dart' show TextRange;

/// [marks] (the words comments are on) moved to fit a text in which the
/// units [start] to [end] were replaced by [inserted] units, until the
/// Rust core says where they are now.
///
/// A mark before the change stays, and one after it moves by the change in
/// length; typing at either edge of one stays outside it. One the change
/// falls inside grows or shrinks with it; one it covers in part loses that
/// part, and one it covers entirely goes.
List<TextRange> shiftMarks(
  List<TextRange> marks, {
  required int start,
  required int end,
  required int inserted,
}) {
  final delta = inserted - (end - start);
  final shifted = <TextRange>[];
  for (final mark in marks) {
    final TextRange moved;
    if (mark.end <= start) {
      moved = mark;
    } else if (mark.start >= end) {
      moved = TextRange(start: mark.start + delta, end: mark.end + delta);
    } else if (mark.start <= start && mark.end >= end && start < end) {
      // The change is inside it.
      moved = TextRange(start: mark.start, end: mark.end + delta);
    } else {
      moved = TextRange(
        start: mark.start < start ? mark.start : start + inserted,
        end: mark.end > end ? mark.end + delta : start,
      );
    }
    if (moved.end > moved.start) {
      shifted.add(moved);
    }
  }
  return shifted;
}
