// Kanso — note_window_event.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'note_window_bloc.dart';

@freezed
sealed class NoteWindowEvent with _$NoteWindowEvent {
  /// The window was opened on the note [noteId]: read it, and keep it read.
  const factory NoteWindowEvent.started(String noteId) = NoteWindowStartedEvent;

  /// The library wrote the note, or the lock session opened or closed: look
  /// at the note again, once the writes of a burst have stopped.
  const factory NoteWindowEvent.lookedAt() = NoteWindowLookedAtEvent;

  /// The person asked to take the note out of the trash.
  const factory NoteWindowEvent.restoreRequested() =
      NoteWindowRestoreRequestedEvent;
}
