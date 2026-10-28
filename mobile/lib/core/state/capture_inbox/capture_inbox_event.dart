// Kanso — capture_inbox_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'capture_inbox_bloc.dart';

@freezed
sealed class CaptureInboxEvent with _$CaptureInboxEvent {
  /// The app started or came back to the front: save what was shared.
  const factory CaptureInboxEvent.saveRequested() =
      CaptureInboxSaveRequestedEvent;

  /// The person has seen the problems: clear them.
  const factory CaptureInboxEvent.problemsDismissed() =
      CaptureInboxProblemsDismissedEvent;
}
