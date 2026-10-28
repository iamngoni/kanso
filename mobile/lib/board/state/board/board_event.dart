// Kanso — board_event.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'board_bloc.dart';

@freezed
sealed class BoardEvent with _$BoardEvent {
  /// The board of [kind] over what [query] lists was opened.
  const factory BoardEvent.started({
    required ListQuery query,
    @Default(BoardKind.notes) BoardKind kind,
  }) = BoardStartedEvent;

  /// The person switched between Notes and Tasks.
  const factory BoardEvent.kindChanged(BoardKind kind) = BoardKindChangedEvent;

  /// The library wrote [ids]: read the board again, once the writes of a
  /// burst have stopped.
  const factory BoardEvent.changesReceived(List<String> ids) =
      BoardChangesReceivedEvent;

  const factory BoardEvent.retried() = BoardRetriedEvent;

  /// The person moved [card] to [lane].
  const factory BoardEvent.cardMoved({
    required CardKey card,
    required BoardLane lane,
  }) = BoardCardMovedEvent;

  const factory BoardEvent.noticeDismissed() = BoardNoticeDismissedEvent;
}
