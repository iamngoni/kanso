// Kanso — send_feedback_event.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'send_feedback_bloc.dart';

@freezed
sealed class SendFeedbackEvent with _$SendFeedbackEvent {
  /// The page opened; [screen] is the fixed screen the person came from.
  const factory SendFeedbackEvent.started({
    @Default('settings') String screen,
  }) = SendFeedbackStartedEvent;

  const factory SendFeedbackEvent.kindChanged(FeedbackKind kind) =
      SendFeedbackKindChangedEvent;

  const factory SendFeedbackEvent.textChanged(String text) =
      SendFeedbackTextChangedEvent;

  const factory SendFeedbackEvent.creditToggled({required bool on}) =
      SendFeedbackCreditToggledEvent;

  const factory SendFeedbackEvent.creditNameChanged(String name) =
      SendFeedbackCreditNameChangedEvent;

  const factory SendFeedbackEvent.appInfoToggled({required bool on}) =
      SendFeedbackAppInfoToggledEvent;

  /// Add screenshot: open the photo picker.
  const factory SendFeedbackEvent.screenshotRequested() =
      SendFeedbackScreenshotRequestedEvent;

  const factory SendFeedbackEvent.screenshotRemoved() =
      SendFeedbackScreenshotRemovedEvent;

  /// Show what's sent was pressed (or pressed again).
  const factory SendFeedbackEvent.sentToggled() = SendFeedbackSentToggledEvent;

  /// Send.
  const factory SendFeedbackEvent.sendRequested() =
      SendFeedbackSendRequestedEvent;
}
