// Kanso — replace_notes_event.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'replace_notes_bloc.dart';

@freezed
sealed class ReplaceNotesEvent with _$ReplaceNotesEvent {
  /// The text to find changed to [text].
  const factory ReplaceNotesEvent.findChanged(String text) =
      ReplaceNotesFindChangedEvent;

  /// The text to put instead changed to [text].
  const factory ReplaceNotesEvent.replacementChanged(String text) =
      ReplaceNotesReplacementChangedEvent;

  /// A toggle was pressed.
  const factory ReplaceNotesEvent.optionToggled(ReplaceOption option) =
      ReplaceNotesOptionToggledEvent;

  /// What is to be found changed and typing has rested: look in the notes
  /// again.
  const factory ReplaceNotesEvent.previewDue() = ReplaceNotesPreviewDueEvent;

  /// The note [noteId] was left out of the replace, or put back in.
  const factory ReplaceNotesEvent.noteToggled(String noteId) =
      ReplaceNotesNoteToggledEvent;

  /// The Replace button was pressed.
  const factory ReplaceNotesEvent.applyRequested() =
      ReplaceNotesApplyRequestedEvent;

  /// Undo replace was pressed.
  const factory ReplaceNotesEvent.undoRequested() =
      ReplaceNotesUndoRequestedEvent;

  /// The line that says what the replace did was dismissed.
  const factory ReplaceNotesEvent.resultDismissed() =
      ReplaceNotesResultDismissedEvent;
}
