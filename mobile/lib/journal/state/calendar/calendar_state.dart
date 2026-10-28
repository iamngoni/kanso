// Kanso — calendar_state.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'calendar_bloc.dart';

@freezed
abstract class CalendarState with _$CalendarState {
  const factory CalendarState({
    @Default(LoadStatus.loading) LoadStatus status,

    /// The month shown.
    @Default(2000) int year,
    @Default(1) int month,

    /// Today, which the grid marks.
    @Default(DayDate(year: 2000, month: 1, day: 1)) DayDate today,

    /// The first day of the week: 0 is Monday.
    @Default(0) int weekStart,

    /// The month as read; null until it is. It keeps showing the last month
    /// read while the next is.
    MonthGrid? grid,
    ApplicationError? error,
  }) = _CalendarState;
}
