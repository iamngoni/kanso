// Kanso — journal_event.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'journal_bloc.dart';

@freezed
sealed class JournalEvent with _$JournalEvent {
  /// Open today's note, made if there is none.
  const factory JournalEvent.todayRequested() = JournalTodayRequestedEvent;

  /// Open the day before or after the daily note [openNote] (today's when
  /// that is not a daily note, or none is open).
  const factory JournalEvent.stepRequested({
    required String? openNote,
    required bool forward,
  }) = JournalStepRequestedEvent;

  /// Open the note of [day], made if there is none.
  const factory JournalEvent.dayRequested(DayDate day) =
      JournalDayRequestedEvent;

  /// Make a note from the template [templateId], in the notebook
  /// [notebookId] (the Inbox when null).
  const factory JournalEvent.fromTemplateRequested({
    required String templateId,
    String? notebookId,
  }) = JournalFromTemplateRequestedEvent;

  /// Make an empty note in the Templates notebook.
  const factory JournalEvent.newTemplateRequested() =
      JournalNewTemplateRequestedEvent;

  /// The person read why it failed.
  const factory JournalEvent.errorDismissed() = JournalErrorDismissedEvent;
}
