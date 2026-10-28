// Kanso — tasks_bloc.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/repos/abs/haptics_repository.dart';
import '../../../core/models/repos/abs/library_repository.dart';
import '../../../core/state/undo_notice/undo_notice_bloc.dart';
import '../../../core/utils/debounce.dart';
import '../../models/data/tasks_notice.dart';
import '../../models/data/tasks_status.dart';

part 'tasks_bloc.freezed.dart';
part 'tasks_event.dart';
part 'tasks_state.dart';

/// How long the writes of a burst may be apart for the tasks to be read
/// again once, after the last.
const Duration tasksDebounce = Duration(milliseconds: 100);

/// The Tasks place: the open tasks of the notes in the desktop's groups,
/// narrowed to a notebook, with done ones if asked, kept up to date as the
/// library changes, and ticked one at a time with the Undo notice.
@injectable
class TasksBloc extends Bloc<TasksEvent, TasksState> {
  TasksBloc(this._library, this._haptics, this._undoNotice)
    : super(const TasksState()) {
    on<TasksStartedEvent>(_onStarted, transformer: restartable());
    on<TasksNotebookChosenEvent>((event, emit) async {
      emit(state.copyWith(notebookId: event.notebookId));
      await _read(emit);
    }, transformer: restartable());
    on<TasksShowDoneChangedEvent>((event, emit) async {
      emit(state.copyWith(showDone: event.showDone));
      await _read(emit);
    }, transformer: restartable());
    on<TasksChangesReceivedEvent>(
      (event, emit) => _read(emit),
      transformer: debounceRestartable<TasksChangesReceivedEvent>(
        tasksDebounce,
      ),
    );
    on<TasksRetriedEvent>((event, emit) => _read(emit));
    on<TasksTickedEvent>(_onTicked, transformer: sequential());
    on<TasksNoticeDismissedEvent>(
      (event, emit) => emit(state.copyWith(notice: null)),
    );
  }

  final LibraryRepository _library;
  final HapticsRepository _haptics;
  final UndoNoticeBloc _undoNotice;
  StreamSubscription<List<String>>? _changes;
  int _reading = 0;

  Future<void> _onStarted(
    TasksStartedEvent event,
    Emitter<TasksState> emit,
  ) async {
    emit(const TasksState());
    await _changes?.cancel();
    _changes = _library.changes().listen(
      (ids) => add(TasksEvent.changesReceived(ids)),
    );
    await _read(emit);
  }

  /// Reads the tasks. A reading that a later one overtook is dropped, so
  /// the state never goes back to older data.
  Future<void> _read(Emitter<TasksState> emit) async {
    final reading = ++_reading;
    final result = await _library.tasks(state.query);
    if (reading != _reading || emit.isDone) {
      return;
    }
    result.fold(
      (error) => emit(state.copyWith(status: TasksStatus.failed, error: error)),
      (board) => emit(
        state.copyWith(status: TasksStatus.ready, board: board, error: null),
      ),
    );
  }

  Future<void> _onTicked(
    TasksTickedEvent event,
    Emitter<TasksState> emit,
  ) async {
    final task = event.task;
    final result = await _library.setTask(
      noteId: task.noteId,
      line: task.line,
      text: task.sourceText,
      checked: !task.checked,
    );
    if (emit.isDone) {
      return;
    }
    await result.fold(
      (error) async => emit(
        state.copyWith(
          notice: error is LockedError
              ? TasksNotice.locked
              : TasksNotice.failed,
        ),
      ),
      (ticked) async {
        switch (ticked) {
          case Ticked_Changed(:final undo, :final message):
            _undoNotice.add(UndoNoticeEvent.shown(undo, message));
            // A task ticked here stays where it was, struck through, until
            // the place is opened again; one cleared goes back to its group.
            final kept = <String>[
              for (final key in state.kept)
                if (key != task.key) key,
              if (!task.checked) task.key,
            ];
            emit(state.copyWith(kept: kept, notice: null));
            await _haptics.selection();
          case Ticked_Unchanged():
            break;
          case Ticked_Missing():
            emit(state.copyWith(notice: TasksNotice.moved));
        }
        await _read(emit);
      },
    );
  }

  @override
  Future<void> close() async {
    await _changes?.cancel();
    return super.close();
  }
}
