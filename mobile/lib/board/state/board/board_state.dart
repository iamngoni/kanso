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
  }) = _BoardState;
}
