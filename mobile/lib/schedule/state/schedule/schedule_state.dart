// Kanso — schedule_state.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'schedule_bloc.dart';

@freezed
abstract class ScheduleState with _$ScheduleState {
  const factory ScheduleState({
    @Default(ScheduleStatus.loading) ScheduleStatus status,
    @Default(ScheduleMode.month) ScheduleMode mode,

    /// A day of the month shown.
    @Default(DayDate(year: 1970, month: 1, day: 1)) DayDate focused,

    /// The day whose items the month view lists.
    @Default(DayDate(year: 1970, month: 1, day: 1)) DayDate selected,

    /// Today, as the page told it.
    @Default(DayDate(year: 1970, month: 1, day: 1)) DayDate today,

    /// The weekday weeks start on: 0 for Monday through 6 for Sunday.
    @Default(0) int weekStart,

    /// Only these notebooks' tasks and notes; every notebook when empty.
    @Default(<String>[]) List<String> books,

    /// Only these date properties place a note; every one when empty.
    @Default(<String>[]) List<String> keys,

    /// Whether done tasks show, as the desktop's Show Completed does.
    @Default(true) bool showDone,

    /// The date properties the notebooks in view define.
    @Default(<String>[]) List<String> dateKeys,

    /// What falls on the days of the month grid.
    @Default(<ScheduleRow>[]) List<ScheduleRow> rows,

    /// Why reading failed, while [status] is failed.
    ApplicationError? error,

    /// What went wrong with the last tick or move, until it is dismissed.
    ScheduleNotice? notice,
  }) = _ScheduleState;

  const ScheduleState._();

  /// Whether any filter narrows the view.
  bool get refined => books.isNotEmpty || keys.isNotEmpty || !showDone;

  /// The rows by their day.
  Map<DayDate, List<ScheduleRow>> get byDay => rowsByDay(rows);

  /// The request that reads [rows].
  ScheduleRequest get request {
    final (from, until) = monthRange(focused, weekStart);
    return ScheduleRequest(
      from: from,
      until: until,
      books: books.isEmpty ? null : books,
      dateKeys: keys.isEmpty ? null : keys,
      includeDone: showDone,
    );
  }
}
