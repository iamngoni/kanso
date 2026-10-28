// Kanso — note_discovery_event.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'note_discovery_bloc.dart';

@freezed
sealed class NoteDiscoveryEvent with _$NoteDiscoveryEvent {
  const factory NoteDiscoveryEvent.started(String noteId) =
      NoteDiscoveryStartedEvent;
  const factory NoteDiscoveryEvent.tagAdded(String tagId) =
      NoteDiscoveryTagAddedEvent;
  const factory NoteDiscoveryEvent.twinDismissed(String otherId) =
      NoteDiscoveryTwinDismissedEvent;
  const factory NoteDiscoveryEvent.notesMerged(String otherId) =
      NoteDiscoveryNotesMergedEvent;
}
