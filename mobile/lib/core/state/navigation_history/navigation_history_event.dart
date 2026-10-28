// Kanso — navigation_history_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'navigation_history_bloc.dart';

@freezed
sealed class NavigationHistoryEvent with _$NavigationHistoryEvent {
  /// The note [noteId] was opened, some way other than back or forward.
  const factory NavigationHistoryEvent.visited(String noteId) =
      NavigationHistoryVisitedEvent;

  /// The person went back.
  const factory NavigationHistoryEvent.backRequested() =
      NavigationHistoryBackRequestedEvent;

  /// The person went forward.
  const factory NavigationHistoryEvent.forwardRequested() =
      NavigationHistoryForwardRequestedEvent;

  /// The screen opened the note the state's `pendingNoteId` named.
  const factory NavigationHistoryEvent.consumed() =
      NavigationHistoryConsumedEvent;

  /// The tabs' BLoC changed the showing tab or its history, or turned the
  /// tabs on or off: the history mirrors it. Sent by the BLoC to itself.
  const factory NavigationHistoryEvent.tabsChanged() =
      NavigationHistoryTabsChangedEvent;
}
