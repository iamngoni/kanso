// Kanso — editor_window.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../models/data/editor_window.dart';

/// How big the editor's windows are, in UTF-16 units.
abstract final class WindowSizes {
  /// The most a window holds, unless a selection is larger. The performance
  /// bench tries other sizes with `--dart-define=KANSO_WINDOW_UNITS=4096`.
  static const int cap = int.fromEnvironment(
    'KANSO_WINDOW_UNITS',
    defaultValue: 8192,
  );

  /// How close to an edge of the window (that is not an edge of the note)
  /// the selection may come before the window moves.
  static const int margin = 1024;
}

/// The window that is all of a note [length] units long.
EditorWindow wholeNote(int length) => EditorWindow(start: 0, end: length);

/// The start of the line that [offset] is on.
int lineStartAt(String note, int offset) {
  if (offset <= 0) {
    return 0;
  }
  final newline = note.lastIndexOf('\n', offset - 1);
  return newline < 0 ? 0 : newline + 1;
}

/// The end of the line that [offset] is on: where its line break is, or the
/// end of the note.
int lineEndAt(String note, int offset) {
  if (offset >= note.length) {
    return note.length;
  }
  final newline = note.indexOf('\n', offset);
  return newline < 0 ? note.length : newline;
}

/// The window of whole lines around the selection [selectionStart] to
/// [selectionEnd] that holds at most [cap] units, with the room left over
/// shared on either side. A selection of more lines than that is held
/// whole, with [margin] units of room beyond each end.
EditorWindow windowAround(
  String note,
  int selectionStart,
  int selectionEnd, {
  int cap = WindowSizes.cap,
  int margin = WindowSizes.margin,
}) {
  final length = note.length;
  final first = lineStartAt(note, selectionStart.clamp(0, length));
  final last = lineEndAt(note, selectionEnd.clamp(0, length));
  final span = last - first;
  if (span + 2 * margin >= cap) {
    // A big selection: it and a margin around it, whole lines outward.
    return EditorWindow(
      start: lineStartAt(note, (first - margin).clamp(0, length)),
      end: lineEndAt(note, (last + margin).clamp(0, length)),
    );
  }
  final room = cap - span;
  final before = room ~/ 2;
  final wantedStart = first - before;
  // Whole lines, inward: the first line start at or after the wanted
  // start, and no later than the selection's.
  var start = 0;
  if (wantedStart > 0) {
    start = lineStartAt(note, wantedStart);
    if (start < wantedStart) {
      final newline = note.indexOf('\n', wantedStart);
      start = newline < 0 || newline + 1 > first ? first : newline + 1;
    }
  }
  final wantedEnd = last + (room - before);
  var end = length;
  if (wantedEnd < length) {
    // Whole lines, inward: the last line end at or before the wanted end,
    // and no earlier than the selection's.
    final newline = note.lastIndexOf('\n', wantedEnd);
    end = newline < last ? last : newline;
  }
  return EditorWindow(start: start, end: end);
}

/// Whether [current] is a good place for the selection, with the note
/// [noteLength] units long: the selection is in it, and not within
/// [margin] units of an edge that has more of the note beyond it.
bool windowFits(
  EditorWindow current,
  int selectionStart,
  int selectionEnd,
  int noteLength, {
  int margin = WindowSizes.margin,
}) {
  if (!current.containsRange(selectionStart, selectionEnd)) {
    return false;
  }
  final roomBefore =
      current.start == 0 || selectionStart - current.start >= margin;
  final roomAfter =
      current.end == noteLength || current.end - selectionEnd >= margin;
  return roomBefore && roomAfter;
}

/// Whether [nextWindow] might have another window to give for the
/// selection, from the lengths alone: the quick question to ask on every
/// keystroke, before the note's text is put together to answer it.
bool needsNewWindow(
  EditorWindow current,
  int selectionStart,
  int selectionEnd,
  int noteLength, {
  int cap = WindowSizes.cap,
  int margin = WindowSizes.margin,
}) {
  final fits = windowFits(
    current,
    selectionStart,
    selectionEnd,
    noteLength,
    margin: margin,
  );
  final span = selectionEnd - selectionStart;
  final oversize = current.length > cap && span + 2 * margin < cap;
  return !fits || oversize;
}

/// The window to hold the selection instead of [current], or null to keep
/// it.
///
/// The window moves when the selection comes within [margin] of an edge
/// that is not the note's, and grows with a selection that outgrows it, up
/// to twice the [cap] (it is the person's to select more, with Select All).
/// A window that grew past [cap] is cut back once the selection is small
/// again.
EditorWindow? nextWindow(
  String note,
  EditorWindow current,
  int selectionStart,
  int selectionEnd, {
  int cap = WindowSizes.cap,
  int margin = WindowSizes.margin,
}) {
  if (!needsNewWindow(
    current,
    selectionStart,
    selectionEnd,
    note.length,
    cap: cap,
    margin: margin,
  )) {
    return null;
  }
  final wanted = windowAround(
    note,
    selectionStart,
    selectionEnd,
    cap: cap,
    margin: margin,
  );
  if (wanted == current) {
    return null;
  }
  // A selection too big to grow to: the window stays.
  if (wanted.length > 2 * cap &&
      current.containsRange(selectionStart, selectionEnd)) {
    return null;
  }
  return wanted;
}
