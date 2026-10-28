// Kanso — note_history_event.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'note_history_bloc.dart';

@freezed
sealed class NoteHistoryEvent with _$NoteHistoryEvent {
  /// The history of the note [noteId] was opened: read it.
  const factory NoteHistoryEvent.started(String noteId) =
      NoteHistoryStartedEvent;

  /// Reading the history again was asked for, after it failed.
  const factory NoteHistoryEvent.retried() = NoteHistoryRetriedEvent;

  /// The version [rev] was chosen.
  const factory NoteHistoryEvent.versionSelected(String rev) =
      NoteHistoryVersionSelectedEvent;

  /// The person switched between what changed and the version itself.
  const factory NoteHistoryEvent.paneChanged(HistoryPane pane) =
      NoteHistoryPaneChangedEvent;

  /// Copy section was pressed on the [index]-th changed section: it is
  /// copied as the version has it.
  const factory NoteHistoryEvent.sectionCopied(int index) =
      NoteHistorySectionCopiedEvent;

  /// Restore this version was pressed.
  const factory NoteHistoryEvent.restoreRequested() =
      NoteHistoryRestoreRequestedEvent;
}
