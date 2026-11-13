// Kanso — schedule_bloc.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/schedule_types.dart';
import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/repos/abs/haptics_repository.dart';
import '../../../core/models/repos/abs/library_repository.dart';
import '../../../core/models/repos/abs/schedule_repository.dart';
import '../../../core/state/undo_notice/undo_notice_bloc.dart';
import '../../../core/utils/debounce.dart';
import '../../models/data/schedule_status.dart';
import '../../utils/schedule_days.dart';

part 'schedule_bloc.freezed.dart';
part 'schedule_event.dart';
part 'schedule_state.dart';

/// How long the writes of a burst may be apart for the Schedule to be read
/// again once, after the last.
const Duration scheduleDebounce = Duration(milliseconds: 100);

/// The Schedule: the tasks and dated notes of a month, as a month grid or an
/// agenda, narrowed by notebook and date property, kept up to date as the
/// library changes. A task is ticked and an item moved to another day one
/// at a time, each with the Undo notice.
@injectable
class ScheduleBloc extends Bloc<ScheduleEvent, ScheduleState> {
  ScheduleBloc(
    this._schedule,
    this._library,
    this._haptics,
    this._undoNotice,
  ) : super(const ScheduleState()) {
    on<ScheduleStartedEvent>(_onStarted, transformer: restartable());
    on<ScheduleModeChangedEvent>(
      (event, emit) => emit(state.copyWith(mode: event.mode)),
    );
    on<ScheduleMonthSteppedEvent>((event, emit) async {
      final next = addMonths(state.focused, event.delta);
      emit(state.copyWith(focused: next, selected: next));
      await _read(emit);
    }, transformer: restartable());
    on<ScheduleTodayShownEvent>((event, emit) async {
      emit(
        state.copyWith(
          today: event.today,
          focused: event.today,
          selected: event.today,
        ),
      );
      await _read(emit);
    }, transformer: restartable());
    on<ScheduleDaySelectedEvent>((event, emit) async {
      final sameMonth =
          event.day.year == state.focused.year &&
          event.day.month == state.focused.month;
      emit(
        state.copyWith(
          selected: event.day,
          focused: sameMonth ? state.focused : event.day,
        ),
      );
      if (!sameMonth) {
        await _read(emit);
      }
    }, transformer: restartable());
    on<ScheduleNotebookToggledEvent>((event, emit) async {
      emit(state.copyWith(books: _toggled(state.books, event.notebookId)));
      await _readAll(emit);
    }, transformer: restartable());
    on<ScheduleNotebooksClearedEvent>((event, emit) async {
      emit(state.copyWith(books: const <String>[]));
      await _readAll(emit);
    }, transformer: restartable());
    on<ScheduleKeyToggledEvent>((event, emit) async {
      emit(state.copyWith(keys: _toggled(state.keys, event.key)));
      await _read(emit);
    }, transformer: restartable());
    on<ScheduleKeysClearedEvent>((event, emit) async {
      emit(state.copyWith(keys: const <String>[]));
      await _read(emit);
    }, transformer: restartable());
    on<ScheduleShowDoneChangedEvent>((event, emit) async {
      emit(state.copyWith(showDone: event.showDone));
      await _read(emit);
    }, transformer: restartable());
    on<ScheduleChangesReceivedEvent>(
      (event, emit) => _read(emit),
      transformer: debounceRestartable<ScheduleChangesReceivedEvent>(
        scheduleDebounce,
      ),
    );
    on<ScheduleRetriedEvent>((event, emit) => _readAll(emit));
    on<ScheduleTickedEvent>(_onTicked, transformer: sequential());
    on<ScheduleMovedEvent>(_onMoved, transformer: sequential());
    on<ScheduleNoticeDismissedEvent>(
      (event, emit) => emit(state.copyWith(notice: null)),
    );
  }

  final ScheduleRepository _schedule;
  final LibraryRepository _library;
  final HapticsRepository _haptics;
  final UndoNoticeBloc _undoNotice;
  StreamSubscription<List<String>>? _changes;
  int _reading = 0;

  static List<String> _toggled(List<String> list, String value) =>
      list.contains(value)
      ? <String>[
          for (final item in list)
            if (item != value) item,
        ]
      : <String>[...list, value];

  Future<void> _onStarted(
    ScheduleStartedEvent event,
    Emitter<ScheduleState> emit,
  ) async {
    emit(
      ScheduleState(
        today: event.today,
        focused: event.today,
        selected: event.today,
        weekStart: event.weekStart,
      ),
    );
    await _changes?.cancel();
    _changes = _library.changes().listen(
      (ids) => add(ScheduleEvent.changesReceived(ids)),
    );
    await _readAll(emit);
  }

  /// Reads the date properties the notebooks in view define, and the days.
  Future<void> _readAll(Emitter<ScheduleState> emit) async {
    final keys = await _schedule.dateKeys(
      state.books.isEmpty ? null : state.books,
    );
    if (emit.isDone) {
      return;
    }
    keys.fold((_) {}, (keys) {
      emit(
        state.copyWith(
          dateKeys: keys,
          keys: <String>[
            for (final key in state.keys)
              if (keys.contains(key)) key,
          ],
        ),
      );
    });
    await _read(emit);
  }

  /// Reads the days. A reading that a later one overtook is dropped, so the
  /// state never goes back to older data.
  Future<void> _read(Emitter<ScheduleState> emit) async {
    final reading = ++_reading;
    final result = await _schedule.schedule(state.request);
    if (reading != _reading || emit.isDone) {
      return;
    }
    result.fold(
      (error) =>
          emit(state.copyWith(status: ScheduleStatus.failed, error: error)),
      (rows) => emit(
        state.copyWith(status: ScheduleStatus.ready, rows: rows, error: null),
      ),
    );
  }

  Future<void> _onTicked(
    ScheduleTickedEvent event,
    Emitter<ScheduleState> emit,
  ) async {
    final row = event.row;
    final line = row.line;
    if (row.kind != ScheduleRowKind.task || line == null) {
      return;
    }
    // A projected occurrence is the real task: its line and words are the
    // task's own, so this ticks it, and a repeating one writes its next.
    final result = await _library.setTask(
      noteId: row.noteId,
      line: line,
      text: row.text,
      checked: !row.done,
    );
    if (emit.isDone) {
      return;
    }
    await result.fold(
      (error) async => emit(state.copyWith(notice: _noticeOf(error))),
      (ticked) async {
        switch (ticked) {
          case Ticked_Changed(:final undo, :final message):
            _undoNotice.add(UndoNoticeEvent.shown(undo, message));
            emit(state.copyWith(notice: null));
            await _haptics.selection();
          case Ticked_Unchanged():
            break;
          case Ticked_Missing():
            emit(state.copyWith(notice: ScheduleNotice.moved));
        }
        await _read(emit);
      },
    );
  }

  Future<void> _onMoved(
    ScheduleMovedEvent event,
    Emitter<ScheduleState> emit,
  ) async {
    final row = event.row;
    if (!canMove(row)) {
      return;
    }
    final result = await _schedule.move(
      noteId: row.noteId,
      line: row.kind == ScheduleRowKind.task ? row.line : null,
      text: row.text,
      property: row.kind == ScheduleRowKind.note ? row.property : null,
      to: event.day,
    );
    if (emit.isDone) {
      return;
    }
    await result.fold(
      (error) async => emit(state.copyWith(notice: _noticeOf(error))),
      (moved) async {
        switch (moved) {
          case CardMoved_Changed(:final undo, :final message):
            if (undo != null) {
              _undoNotice.add(
                UndoNoticeEvent.shown(undo, message ?? ''),
              );
            }
            emit(state.copyWith(notice: null));
            await _haptics.selection();
          case CardMoved_Unchanged():
            break;
          case CardMoved_Missing():
            emit(state.copyWith(notice: ScheduleNotice.moved));
        }
        await _read(emit);
      },
    );
  }

  static ScheduleNotice _noticeOf(ApplicationError error) =>
      error is LockedError ? ScheduleNotice.locked : ScheduleNotice.failed;

  @override
  Future<void> close() async {
    await _changes?.cancel();
    return super.close();
  }
}
