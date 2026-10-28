// Kanso — ask_update.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import 'ai_note_ref.dart';

part 'ask_update.freezed.dart';

/// What the core tells the screen while a question is answered.
///
/// An answer is Markdown. A citation in it is a link to
/// `kanso://open/note/<id>` (see `noteCitationHref`), so the screen can open
/// the note without knowing how the core found it.
@freezed
sealed class AskUpdate with _$AskUpdate {
  /// The notes are being searched.
  const factory AskUpdate.searching() = AskSearching;

  /// The notes are read and the model is working.
  const factory AskUpdate.thinking() = AskThinking;

  /// The notes the answer is being written from.
  const factory AskUpdate.usedNotes({required List<AiNoteRef> notes}) =
      AskUsedNotes;

  /// The answer written so far: all of it, not only the new part.
  const factory AskUpdate.answer({required String text}) = AskAnswer;

  /// The answer is complete.
  const factory AskUpdate.done({
    required String answer,
    required List<AiNoteRef> used,
  }) = AskDone;

  /// No note matches the question.
  const factory AskUpdate.nothingMatches() = AskNothingMatches;

  /// The model stopped before it finished; the answer so far is kept.
  const factory AskUpdate.cutOff() = AskCutOff;

  /// The question could not be answered, in [message]'s plain words.
  const factory AskUpdate.error({required String message}) = AskFailed;
}
