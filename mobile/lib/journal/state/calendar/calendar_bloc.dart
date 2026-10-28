// Kanso — calendar_bloc.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/models/repos/abs/journal_repository.dart';

part 'calendar_bloc.freezed.dart';
part 'calendar_event.dart';
part 'calendar_state.dart';

/// The month calendar of the journal: the month shown, with the days that
/// have a note marked and today marked, stepped back and forward, and put
/// back on today's month.
@injectable
class CalendarBloc extends Bloc<CalendarEvent, CalendarState> {
  CalendarBloc(this._journal) : super(const CalendarState()) {
    on<CalendarStartedEvent>(_onStarted, transformer: restartable());
    on<CalendarMonthSteppedEvent>(_onStepped, transformer: restartable());
    on<CalendarTodayShownEvent>(_onToday, transformer: restartable());
    on<CalendarRetriedEvent>(
      (event, emit) => _read(emit),
      transformer: restartable(),
    );
  }

  final JournalRepository _journal;

  Future<void> _onStarted(
    CalendarStartedEvent event,
    Emitter<CalendarState> emit,
  ) async {
    // The month of the daily note that is open, or today's.
    final openNote = event.openNote;
    final around =
        (openNote == null ? null : await _journal.dailyDate(openNote)) ??
        event.today;
    if (emit.isDone) {
      return;
    }
    emit(
      CalendarState(
        today: event.today,
        weekStart: event.weekStart,
        year: around.year,
        month: around.month,
      ),
    );
    await _read(emit);
  }

  Future<void> _onStepped(
    CalendarMonthSteppedEvent event,
    Emitter<CalendarState> emit,
  ) {
    // Months are counted from year 0 so a step over a year's end is plain
    // arithmetic.
    final index = state.year * 12 + (state.month - 1) + event.by;
    emit(state.copyWith(year: index ~/ 12, month: index % 12 + 1));
    return _read(emit);
  }

  Future<void> _onToday(
    CalendarTodayShownEvent event,
    Emitter<CalendarState> emit,
  ) {
    emit(state.copyWith(year: state.today.year, month: state.today.month));
    return _read(emit);
  }

  Future<void> _read(Emitter<CalendarState> emit) async {
    final year = state.year;
    final month = state.month;
    final result = await _journal.month(
      year: year,
      month: month,
      weekStart: state.weekStart,
      today: state.today,
    );
    if (emit.isDone || year != state.year || month != state.month) {
      return;
    }
    result.fold(
      (error) => emit(state.copyWith(status: LoadStatus.failed, error: error)),
      (grid) => emit(
        state.copyWith(status: LoadStatus.ready, grid: grid, error: null),
      ),
    );
  }
}
