// Kanso — ask_turn.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import '../../../core/models/data/ai_note_ref.dart';
import '../../../core/models/data/ask_closest.dart';
import '../../../core/models/data/ask_receipt.dart';
import '../../../core/models/data/ask_scope.dart';
import '../../../core/models/data/ask_time_note.dart';
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

    /// What the question searched.
    @Default(AskScope.everything) AskScope scope,

    /// The lines the answer's sentences rest on, checked against what was
    /// sent; the Sources list.
    @Default(<AskReceipt>[]) List<AskReceipt> receipts,

    /// Notes it read but did not cite.
    @Default(<AiNoteRef>[]) List<AiNoteRef> alsoRead,

    /// The banner over the answer; empty when there is none.
    @Default('') String banner,

    /// How many sentences are marked as not found.
    @Default(0) int unverified,

    /// How many sentences cite nothing in an answer that cites elsewhere.
    @Default(0) int noSource,

    /// Said above the question when the conversation before it was ended.
    @Default('') String notice,

    /// The notes do not hold the answer: "You never wrote that down."
    @Default(false) bool neverWrote,

    /// The closest notes, shown with [neverWrote].
    @Default(<AskClosest>[]) List<AskClosest> closest,

    /// The time the question was understood to be about.
    AskTimeNote? time,

    /// Nothing matched in a scope narrower than all notes.
    @Default(false) bool narrow,

    /// The note the answer was saved as, once it was.
    String? savedNoteId,
  }) = _AskTurn;
}
