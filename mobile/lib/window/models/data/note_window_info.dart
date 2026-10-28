// Kanso — note_window_info.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'note_window_info.freezed.dart';

/// A note window that is open: which note, and the name its window has in
/// the system's window list (the note's title; "Untitled"; "Locked note").
@freezed
abstract class NoteWindowInfo with _$NoteWindowInfo {
  const factory NoteWindowInfo({
    required String noteId,
    @Default('') String title,
  }) = _NoteWindowInfo;
}
