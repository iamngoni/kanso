// Kanso — undo_notice_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'undo_notice_bloc.dart';

@freezed
sealed class UndoNoticeEvent with _$UndoNoticeEvent {
  /// A change was made that can be taken back: show [message] with Undo,
  /// in place of any notice showing, for as long as the notice stays.
  const factory UndoNoticeEvent.shown(int ticket, String message) =
      UndoNoticeShownEvent;

  /// The person pressed Undo.
  const factory UndoNoticeEvent.undoPressed() = UndoNoticePressedEvent;

  /// The notice should go (the person went elsewhere).
  const factory UndoNoticeEvent.dismissed() = UndoNoticeDismissedEvent;

  /// The time of the notice numbered [serial] is up.
  const factory UndoNoticeEvent.timedOut(int serial) = UndoNoticeTimedOutEvent;

  /// The person has read why an undo failed.
  const factory UndoNoticeEvent.errorDismissed() =
      UndoNoticeErrorDismissedEvent;
}
