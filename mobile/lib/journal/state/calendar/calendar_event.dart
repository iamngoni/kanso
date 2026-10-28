// Kanso — calendar_event.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'calendar_bloc.dart';

@freezed
sealed class CalendarEvent with _$CalendarEvent {
  /// The calendar was opened on the month of the open daily note
  /// [openNote] (today's month when it is none, or not a daily note), with
  /// weeks that start on [weekStart] (0 is Monday).
  const factory CalendarEvent.started({
    required String? openNote,
    required DayDate today,
    required int weekStart,
  }) = CalendarStartedEvent;

  /// Show the month [by] months from the one shown (-1 the one before).
  const factory CalendarEvent.monthStepped(int by) = CalendarMonthSteppedEvent;

  /// Show today's month.
  const factory CalendarEvent.todayShown() = CalendarTodayShownEvent;

  /// Try reading the month again.
  const factory CalendarEvent.retried() = CalendarRetriedEvent;
}
