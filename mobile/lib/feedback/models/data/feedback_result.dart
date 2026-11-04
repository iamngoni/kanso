// Kanso — feedback_result.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'feedback_result.freezed.dart';

/// How sending a report ended.
enum FeedbackOutcome {
  /// The server has it.
  sent,

  /// Saved on the phone; it goes when the phone is back online.
  queued,

  /// Feedback is paused on the server; the report is saved.
  paused,

  /// Too many today; the report is saved and waits.
  rateLimited,

  /// The server refused a field; nothing was kept.
  rejected,

  /// The report can't be sent as written ([FeedbackProblem]).
  invalid,
}

/// Why a report can't be sent as written.
enum FeedbackProblem {
  tooShort,
  tooLong,
  badCharacter,
  badCredit,
  badScreenshot,
  other,
}

/// The result of Send, with what the screen needs to say it.
@freezed
abstract class FeedbackSendResult with _$FeedbackSendResult {
  const factory FeedbackSendResult({
    required FeedbackOutcome outcome,

    /// The JSON path the server refused, for [FeedbackOutcome.rejected].
    String? field,

    /// Why it can't be sent, for [FeedbackOutcome.invalid].
    FeedbackProblem? problem,
  }) = _FeedbackSendResult;
}

/// "Show what's sent": the exact JSON, or why there is none yet.
@freezed
abstract class FeedbackPayload with _$FeedbackPayload {
  const factory FeedbackPayload({
    @Default('') String json,
    FeedbackProblem? problem,
  }) = _FeedbackPayload;
}
