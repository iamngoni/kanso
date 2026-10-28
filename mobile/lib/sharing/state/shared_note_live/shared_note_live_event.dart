// Kanso — shared_note_live_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'shared_note_live_bloc.dart';

@freezed
sealed class SharedNoteLiveEvent with _$SharedNoteLiveEvent {
  /// The note [noteId] was opened: join its room if it is shared.
  const factory SharedNoteLiveEvent.opened(String noteId) =
      SharedNoteLiveOpenedEvent;

  /// The app came back to the front: join the room again.
  const factory SharedNoteLiveEvent.resumed() = SharedNoteLiveResumedEvent;
}
