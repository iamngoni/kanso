// Kanso — schedule_event.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'schedule_bloc.dart';

@freezed
sealed class ScheduleEvent with _$ScheduleEvent {
  /// The Schedule was opened on [today]'s month, with weeks starting on
  /// [weekStart] (0 for Monday through 6 for Sunday): read it and keep it
  /// read.
  const factory ScheduleEvent.started({
    required DayDate today,
    required int weekStart,
  }) = ScheduleStartedEvent;

  /// The person chose month or agenda.
  const factory ScheduleEvent.modeChanged(ScheduleMode mode) =
      ScheduleModeChangedEvent;

  /// The person stepped [delta] months.
  const factory ScheduleEvent.monthStepped(int delta) =
      ScheduleMonthSteppedEvent;

  /// The person went back to today.
  const factory ScheduleEvent.todayShown(DayDate today) =
      ScheduleTodayShownEvent;

  /// The person chose a day of the month.
  const factory ScheduleEvent.daySelected(DayDate day) =
      ScheduleDaySelectedEvent;

  /// The person turned [notebookId] on or off in the notebook filter.
  const factory ScheduleEvent.notebookToggled(String notebookId) =
      ScheduleNotebookToggledEvent;

  /// The person asked for every notebook.
  const factory ScheduleEvent.notebooksCleared() =
      ScheduleNotebooksClearedEvent;

  /// The person turned the date property [key] on or off in the filter.
  const factory ScheduleEvent.keyToggled(String key) = ScheduleKeyToggledEvent;

  /// The person asked for every date property.
  const factory ScheduleEvent.keysCleared() = ScheduleKeysClearedEvent;

  /// The person turned showing done tasks on or off.
  const factory ScheduleEvent.showDoneChanged({required bool showDone}) =
      ScheduleShowDoneChangedEvent;

  /// The library wrote [ids]: read again, once the writes of a burst have
  /// stopped.
  const factory ScheduleEvent.changesReceived(List<String> ids) =
      ScheduleChangesReceivedEvent;

  /// The person asked to try again after a failure.
  const factory ScheduleEvent.retried() = ScheduleRetriedEvent;

  /// The person ticked [row], or cleared it if it was ticked. A projected
  /// occurrence ticks the real task.
  const factory ScheduleEvent.ticked(ScheduleRow row) = ScheduleTickedEvent;

  /// The person moved [row] to [day].
  const factory ScheduleEvent.moved(ScheduleRow row, DayDate day) =
      ScheduleMovedEvent;

  /// The person has read what went wrong.
  const factory ScheduleEvent.noticeDismissed() = ScheduleNoticeDismissedEvent;
}
