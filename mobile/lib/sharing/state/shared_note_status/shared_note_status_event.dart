// Kanso — shared_note_status_event.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'shared_note_status_bloc.dart';

@freezed
sealed class SharedNoteStatusEvent with _$SharedNoteStatusEvent {
  /// The note [noteId] is open: read its status, and follow it.
  const factory SharedNoteStatusEvent.started(String noteId) =
      SharedNoteStatusStartedEvent;

  /// The room or a sync said the note changed: read it again.
  const factory SharedNoteStatusEvent.refreshed() =
      SharedNoteStatusRefreshedEvent;

  /// The banner's Dismiss: forget what couldn't be done.
  const factory SharedNoteStatusEvent.problemsDismissed() =
      SharedNoteStatusProblemsDismissedEvent;
}
