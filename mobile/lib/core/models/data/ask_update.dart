// Kanso — ask_update.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import 'ai_note_ref.dart';
import 'ask_closest.dart';
import 'ask_receipt.dart';
import 'ask_time_note.dart';

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

  /// The time the question was understood to be about.
  const factory AskUpdate.time({required AskTimeNote time}) = AskTime;

  /// The answer is complete and its receipts were checked.
  const factory AskUpdate.done({
    required String answer,
    required List<AiNoteRef> used,
    @Default(<AskReceipt>[]) List<AskReceipt> receipts,

    /// Notes it read but did not cite.
    @Default(<AiNoteRef>[]) List<AiNoteRef> alsoRead,

    /// The banner over the answer; empty when there is none.
    @Default('') String banner,

    /// How many sentences are marked as not found.
    @Default(0) int unverified,

    /// The notes do not hold the answer.
    @Default(false) bool neverWrote,
    @Default(<AskClosest>[]) List<AskClosest> closest,
    AskTimeNote? time,
  }) = AskDone;

  /// No note matches the question. [narrow] says the scope was narrower than
  /// all notes, so searching all may help.
  const factory AskUpdate.nothingMatches({@Default(false) bool narrow}) =
      AskNothingMatches;

  /// The model stopped before it finished; the answer so far is kept.
  const factory AskUpdate.cutOff() = AskCutOff;

  /// The question could not be answered, in [message]'s plain words.
  const factory AskUpdate.error({required String message}) = AskFailed;
}
