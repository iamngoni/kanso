// Kanso — ai_note_ref.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'ai_note_ref.freezed.dart';

/// A note an answer was written from: enough to list it and open it.
@freezed
abstract class AiNoteRef with _$AiNoteRef {
  const factory AiNoteRef({
    required String id,

    /// The note's title; empty for an untitled note.
    @Default('') String title,
  }) = _AiNoteRef;
}
