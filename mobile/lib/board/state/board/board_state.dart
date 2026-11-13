// Kanso — board_state.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'board_bloc.dart';

/// Why the last move did not happen the way it was asked.
@freezed
sealed class BoardNotice with _$BoardNotice {
  /// The write failed.
  const factory BoardNotice.failed(ApplicationError error) = BoardNoticeFailed;

  /// The task changed in its note meanwhile (or the note went); the board
  /// has been read again.
  const factory BoardNotice.changedInNote() = BoardNoticeChangedInNote;

  /// A change to the notebook's board was written; [message] is the core's
  /// words for it, and [undo] the ticket that takes it back.
  const factory BoardNotice.done({required String message, int? undo}) =
      BoardNoticeDone;
}

/// A lane being removed that still has notes: where they go is asked.
@freezed
abstract class BoardRemoval with _$BoardRemoval {
  const factory BoardRemoval({
    required String lane,
    required LaneRemoval plan,

    /// The lane the notes go to; null is "no value".
    String? target,
  }) = _BoardRemoval;
}

@freezed
abstract class BoardState with _$BoardState {
  const factory BoardState({
    /// The list the board lays out; null until started.
    ListQuery? query,
    @Default(BoardKind.notes) BoardKind kind,
    @Default(LoadStatus.loading) LoadStatus status,

    /// The columns, as last read.
    BoardView? data,

    /// Why reading failed, while [status] is failed.
    ApplicationError? error,

    /// Why the last move did not happen, until it is dismissed.
    BoardNotice? notice,

    /// The lane the settings are asking about, while they are.
    BoardRemoval? removal,

    /// Why the last change in the settings was refused, in the core's words.
    String? settingsError,
  }) = _BoardState;
}
