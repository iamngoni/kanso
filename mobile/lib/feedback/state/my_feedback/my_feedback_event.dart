// Kanso — my_feedback_event.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'my_feedback_bloc.dart';

@freezed
sealed class MyFeedbackEvent with _$MyFeedbackEvent {
  /// Settings opened (the reports on the phone, nothing asked) or the app
  /// launched (asked, at most every 6 hours).
  const factory MyFeedbackEvent.started({@Default(false) bool launch}) =
      MyFeedbackStartedEvent;

  /// My feedback opened or was pulled down: send what is waiting and ask
  /// the server how the reports stand.
  const factory MyFeedbackEvent.refreshRequested() =
      MyFeedbackRefreshRequestedEvent;

  /// Delete report [id].
  const factory MyFeedbackEvent.deleteRequested(String id) =
      MyFeedbackDeleteRequestedEvent;

  /// The released reports were looked at.
  const factory MyFeedbackEvent.seen() = MyFeedbackSeenEvent;

  /// The app came back to the front, or the phone is back online.
  const factory MyFeedbackEvent.resumed() = MyFeedbackResumedEvent;
}
