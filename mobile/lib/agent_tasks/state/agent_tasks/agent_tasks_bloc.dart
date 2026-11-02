// Kanso — agent_tasks_bloc.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/models/data/agent_task.dart';
import '../../../core/models/data/agent_task_prefs.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/models/repos/abs/agent_task_notifier.dart';
import '../../../core/models/repos/abs/agent_tasks_repository.dart';

part 'agent_tasks_bloc.freezed.dart';
part 'agent_tasks_event.dart';
part 'agent_tasks_state.dart';

/// How often the list of tasks is asked for while it is open.
const Duration agentTasksPoll = Duration(seconds: 1);

/// Agent tasks: starting one, watching it work, answering its question and
/// stopping it. Every event goes through one sequential handler, so a poll
/// never lands between a start and the first list, and the list the person
/// sees is always the newest one.
@injectable
class AgentTasksBloc extends Bloc<AgentTasksEvent, AgentTasksState> {
  AgentTasksBloc(this._tasks, this._notifier) : super(const AgentTasksState()) {
    on<AgentTasksEvent>(_handle, transformer: sequential());
  }

  final AgentTasksRepository _tasks;
  final AgentTaskNotifier _notifier;
  Timer? _timer;

  Future<void> _handle(
    AgentTasksEvent event,
    Emitter<AgentTasksState> emit,
  ) => switch (event) {
    AgentTasksStartedEvent() => _onStarted(emit),
    AgentTasksPolledEvent() => _refresh(emit),
    AgentTasksGoalSentEvent() => _onGoalSent(event, emit),
    AgentTasksStoppedEvent() => _then(_tasks.stop(event.id), emit),
    AgentTasksRepliedEvent() => _then(
      _tasks.reply(event.id, event.text),
      emit,
    ),
    AgentTasksDismissedEvent() => _then(_tasks.discard(event.id), emit),
    AgentTasksTroubleClearedEvent() => Future<void>.sync(
      () => emit(state.copyWith(trouble: null)),
    ),
  };

  Future<void> _onStarted(Emitter<AgentTasksState> emit) async {
    final prefs = (await _tasks.prefs()).toOption().fold(
      () => const AgentTaskPrefs(),
      (found) => found,
    );
    emit(state.copyWith(prefs: prefs));
    await _refresh(emit);
    _timer?.cancel();
    _timer = Timer.periodic(
      agentTasksPoll,
      (_) => add(const AgentTasksEvent.polled()),
    );
  }

  Future<void> _onGoalSent(
    AgentTasksGoalSentEvent event,
    Emitter<AgentTasksState> emit,
  ) async {
    emit(state.copyWith(starting: true, trouble: null));
    final result = await _tasks.start(
      goal: event.goal,
      noteIds: event.noteIds,
      newNoteBook: event.newNoteBook,
      confirmedAll: event.confirmedAll,
    );
    result.fold(
      (error) => emit(state.copyWith(starting: false, trouble: error.message)),
      (_) => emit(state.copyWith(starting: false, started: state.started + 1)),
    );
    await _refresh(emit);
  }

  Future<void> _then(
    Future<Object> call,
    Emitter<AgentTasksState> emit,
  ) async {
    await call;
    await _refresh(emit);
  }

  Future<void> _refresh(Emitter<AgentTasksState> emit) async {
    final result = await _tasks.tasks();
    result.fold(
      (_) => emit(state.copyWith(status: LoadStatus.failed)),
      (rows) {
        _announce(state.tasks, rows);
        emit(state.copyWith(status: LoadStatus.ready, tasks: rows));
      },
    );
  }

  /// Tells the person, when the app is behind other apps, of a task that
  /// stopped to be reviewed or answered.
  void _announce(List<AgentTask> before, List<AgentTask> now) {
    for (final task in now) {
      final was = before.where((t) => t.id == task.id).firstOrNull;
      if (was != null &&
          was.active &&
          (task.reviewable || task.phase == AgentTaskPhase.asking)) {
        unawaited(_notifier.paused(id: task.id));
      }
    }
  }

  @override
  Future<void> close() {
    _timer?.cancel();
    return super.close();
  }
}
