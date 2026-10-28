// Kanso — answer_span.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'answer_span.freezed.dart';

/// A run of an answer's text and what it does.
@freezed
sealed class AnswerSpan with _$AnswerSpan {
  /// Text, plain or marked up.
  const factory AnswerSpan.text(
    String text, {
    @Default(false) bool bold,
    @Default(false) bool italic,
    @Default(false) bool code,
  }) = AnswerText;

  /// A citation: a tap opens the note [noteId].
  const factory AnswerSpan.note({
    required String text,
    required String noteId,
  }) = AnswerNoteLink;

  /// A link to a page on the web; only `http` and `https` are links.
  const factory AnswerSpan.web({required String text, required String url}) =
      AnswerWebLink;
}
