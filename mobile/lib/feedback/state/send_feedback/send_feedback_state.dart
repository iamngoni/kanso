// Kanso — send_feedback_state.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'send_feedback_bloc.dart';

@freezed
abstract class SendFeedbackState with _$SendFeedbackState {
  const factory SendFeedbackState({
    @Default(FeedbackDraft()) FeedbackDraft draft,

    /// The fixed screen the person came from.
    @Default('settings') String screen,

    /// Whether Show what's sent is open.
    @Default(false) bool showSent,

    /// What a send would carry, while Show what's sent is open.
    @Default(FeedbackPayload()) FeedbackPayload payload,

    /// The report is being saved and sent.
    @Default(false) bool sending,

    /// The photo that was picked is over 5 MB.
    @Default(false) bool screenshotTooBig,

    /// How the last send ended; null before one. A report that was sent or
    /// saved closes the page; a refusal stays on it.
    FeedbackSendResult? result,
  }) = _SendFeedbackState;
}
