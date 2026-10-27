// Kanso — note_creation_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'note_creation_bloc.dart';

@freezed
sealed class NoteCreationEvent with _$NoteCreationEvent {
  /// The person asked for a new note from the list of [place]; [pinned] when
  /// that list shows only pinned notes. It opens at [start].
  const factory NoteCreationEvent.requested(
    ListPlace place, {
    @Default(false) bool pinned,
    @Default(NewNoteStart.title) NewNoteStart start,
  }) = NoteCreationRequestedEvent;

  /// The person asked to save what they pick from [source] (a photo, the
  /// photo library, a PDF) as a new note in the Inbox.
  const factory NoteCreationEvent.captured(MediaSource source) =
      NoteCreationCapturedEvent;

  /// A link in a note asked for the note titled [title], which is not there.
  const factory NoteCreationEvent.titled(String title) =
      NoteCreationTitledEvent;

  /// The screen has gone to the note the state named.
  const factory NoteCreationEvent.openingHandled() =
      NoteCreationOpeningHandledEvent;

  /// The person has read why no note was made.
  const factory NoteCreationEvent.noticeDismissed() =
      NoteCreationNoticeDismissedEvent;
}
