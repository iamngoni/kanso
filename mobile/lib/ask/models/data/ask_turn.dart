// Kanso — ask_turn.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import '../../../core/models/data/ai_note_ref.dart';
import 'ask_outcome.dart';

part 'ask_turn.freezed.dart';

/// One question in the conversation and its answer, so far or whole.
@freezed
abstract class AskTurn with _$AskTurn {
  const factory AskTurn({
    required String question,

    /// The answer as Markdown, with citations as links to notes.
    @Default('') String answer,

    /// The notes the answer was written from.
    @Default(<AiNoteRef>[]) List<AiNoteRef> used,
    @Default(AskOutcome.pending) AskOutcome outcome,

    /// Why the question failed, in plain words, while [outcome] is failed.
    String? failure,
  }) = _AskTurn;
}
