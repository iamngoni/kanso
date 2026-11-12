// Kanso — agent_session_bloc.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/models/data/agent_session.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/models/repos/abs/agent_tasks_repository.dart';

part 'agent_session_bloc.freezed.dart';
part 'agent_session_event.dart';
part 'agent_session_state.dart';

/// How often the transcript is asked for while the session is open.
const Duration agentSessionPoll = Duration(seconds: 1);

/// One agent session open on screen: its transcript, kept fresh by polling,
/// and what the person does in it. Every event goes through one sequential
/// handler, so a poll never lands between a send and the transcript that
/// shows it.
@injectable
class AgentSessionBloc extends Bloc<AgentSessionEvent, AgentSessionState> {
  AgentSessionBloc(this._tasks) : super(const AgentSessionState()) {
    on<AgentSessionEvent>(_handle, transformer: sequential());
  }

  final AgentTasksRepository _tasks;
  String _key = '';
  Timer? _timer;

  Future<void> _handle(
    AgentSessionEvent event,
    Emitter<AgentSessionState> emit,
  ) => switch (event) {
    AgentSessionStartedEvent() => _onStarted(event.key, emit),
    AgentSessionPolledEvent() => _refresh(emit),
    AgentSessionFollowUpSentEvent() => _onFollowUp(event.text, emit),
    AgentSessionStoppedEvent() => _onStopped(emit),
    AgentSessionEndedEvent() => _then(_tasks.endSession(_key), emit),
    AgentSessionRenamedEvent() => _then(
      _tasks.renameSession(_key, event.title),
      emit,
    ),
    AgentSessionDeletedEvent() => _onDeleted(emit),
    AgentSessionDismissedEvent() => _onDismissed(emit),
    AgentSessionUndoneEvent() => _onUndone(emit),
    AgentSessionTroubleClearedEvent() => Future<void>.sync(
      () => emit(state.copyWith(trouble: null)),
    ),
  };

  Future<void> _onStarted(String key, Emitter<AgentSessionState> emit) async {
    _key = key;
    await _refresh(emit);
    _timer?.cancel();
    _timer = Timer.periodic(
      agentSessionPoll,
      (_) => add(const AgentSessionEvent.polled()),
    );
  }

  Future<void> _onFollowUp(
    String text,
    Emitter<AgentSessionState> emit,
  ) async {
    emit(state.copyWith(sending: true, trouble: null));
    final result = await _tasks.followUp(_key, text);
    emit(
      state.copyWith(
        sending: false,
        trouble: result.fold((error) => error.message, (_) => null),
      ),
    );
    await _refresh(emit);
  }

  Future<void> _onStopped(Emitter<AgentSessionState> emit) async {
    final id = state.transcript?.session.id;
    if (id != null) {
      await _tasks.stop(id);
    }
    await _refresh(emit);
  }

  Future<void> _onDeleted(Emitter<AgentSessionState> emit) async {
    await _tasks.deleteSession(_key);
    await _refresh(emit);
  }

  Future<void> _onDismissed(Emitter<AgentSessionState> emit) async {
    final id = state.transcript?.session.id;
    if (id != null) {
      await _tasks.discard(id);
    }
    await _refresh(emit);
  }

  Future<void> _onUndone(Emitter<AgentSessionState> emit) async {
    final id = state.transcript?.session.id;
    if (id != null) {
      final result = await _tasks.undo(id);
      emit(
        state.copyWith(
          trouble: result.fold((error) => error.message, (_) => null),
        ),
      );
    }
    await _refresh(emit);
  }

  Future<void> _then(
    Future<Object> call,
    Emitter<AgentSessionState> emit,
  ) async {
    await call;
    await _refresh(emit);
  }

  Future<void> _refresh(Emitter<AgentSessionState> emit) async {
    final result = await _tasks.transcript(_key);
    result.fold(
      (error) => emit(
        error is NotFoundError
            ? state.copyWith(status: LoadStatus.ready, gone: true)
            : state.copyWith(status: LoadStatus.failed),
      ),
      (found) => emit(
        state.copyWith(
          status: LoadStatus.ready,
          transcript: found,
          gone: false,
        ),
      ),
    );
  }

  @override
  Future<void> close() {
    _timer?.cancel();
    return super.close();
  }
}
