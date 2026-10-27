// Kanso — editor_window.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'editor_window.freezed.dart';

/// The part of a note the editor's field holds: whole lines from [start] to
/// [end], in UTF-16 units of the note.
///
/// A note short enough to lay out at once is one window. A long one is
/// edited a window at a time (the text-layout cost of every keystroke
/// grows with the text the field holds); the rest is drawn read-only
/// before and after it.
@freezed
abstract class EditorWindow with _$EditorWindow {
  const factory EditorWindow({required int start, required int end}) =
      _EditorWindow;

  const EditorWindow._();

  /// How many units the window holds.
  int get length => end - start;

  /// Whether [offset] of the note is in the window, at either edge too.
  bool contains(int offset) => offset >= start && offset <= end;

  /// The text of the window in [note].
  String slice(String note) => note.substring(start, end);

  /// Whether [first] to [last] is in the window.
  bool containsRange(int first, int last) => first >= start && last <= end;
}
