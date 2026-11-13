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

  /// The person chose what the notebook's board is grouped by: a select
  /// property, or null for the status lanes.
  const factory BoardEvent.groupChanged(String? by) = BoardGroupChangedEvent;

  /// The person turned the lane of cards without a value on or off.
  const factory BoardEvent.showUnsetChanged({required bool show}) =
      BoardShowUnsetChangedEvent;

  /// The person renamed, moved or added a lane, or removed one with no
  /// notes in it.
  const factory BoardEvent.laneEdited(LaneChange change) = BoardLaneEditedEvent;

  /// The person asked to remove [lane]: it goes at once when it is empty,
  /// else the board asks where its notes go.
  const factory BoardEvent.laneRemovalRequested(String lane) =
      BoardLaneRemovalRequestedEvent;

  /// The person chose where the notes of the lane being removed go; null is
  /// "no value".
  const factory BoardEvent.removalTargetChosen(String? target) =
      BoardRemovalTargetChosenEvent;

  const factory BoardEvent.removalConfirmed() = BoardRemovalConfirmedEvent;

  const factory BoardEvent.removalCancelled() = BoardRemovalCancelledEvent;

  /// The person asked to take back the change kept under [ticket].
  const factory BoardEvent.undone(int ticket) = BoardUndoneEvent;

  /// The settings were closed: what they last said is dropped.
  const factory BoardEvent.settingsClosed() = BoardSettingsClosedEvent;
}
