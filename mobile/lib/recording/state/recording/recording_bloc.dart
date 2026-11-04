// Kanso — recording_bloc.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/recording_types.dart';
import '../../../core/models/data/application_error.dart';
import '../../models/repos/abs/recording_repository.dart';

part 'recording_bloc.freezed.dart';
part 'recording_event.dart';
part 'recording_state.dart';

/// Recording a note from the phone's microphone (issue #38): the bar that
/// follows the app, Pause, Stop and Discard, and the recovery offer after a
/// crash.
///
/// One recording runs at a time, for the whole app, so this is a singleton:
/// leaving the screen does not stop it. Every event runs in turn, so a Stop
/// never races the tick that reads the clock, and the recovery check at
/// launch never races the first Start.
@lazySingleton
class RecordingBloc extends Bloc<RecordingEvent, RecordingState> {
  RecordingBloc(
    this._repository, {
    // The tick is a named, public argument so tests can slow it down. Not
    // injected: no Duration is registered, and asking for one throws.
    @ignoreParam Duration tick = const Duration(milliseconds: 250),
    // ignore: prefer_initializing_formals
  }) : _tick = tick,
       super(const RecordingState()) {
    on<RecordingEvent>(_onEvent, transformer: sequential());
  }

  final RecordingRepository _repository;
  final Duration _tick;
  RecordingSession? _session;
  Timer? _timer;

  Future<void> _onEvent(
    RecordingEvent event,
    Emitter<RecordingState> emit,
  ) async {
    switch (event) {
      case RecordingStartedEvent(:final noteId):
        await _start(noteId, emit);
      case RecordingTickedEvent():
        await _onTick(emit);
      case RecordingPausedEvent():
        _session?.pause();
        emit(state.copyWith(paused: true));
      case RecordingResumedEvent():
        _session?.resume();
        emit(state.copyWith(paused: false));
      case RecordingStoppedEvent():
        await _stop(emit);
      case RecordingDiscardedEvent():
        await _discard(emit);
      case RecordingFoundCheckedEvent():
        final found = await _repository.found();
        emit(state.copyWith(found: found.getOrElse(() => const [])));
      case RecordingRecoveredEvent(:final id):
        await _recover(id, emit);
      case RecordingFoundDiscardedEvent(:final id):
        await _repository.discardFound(id);
        emit(
          state.copyWith(
            found: <RecordingFound>[
              for (final f in state.found)
                if (f.id != id) f,
            ],
          ),
        );
      case RecordingNoticeClearedEvent():
        emit(state.copyWith(problem: null, saved: null, ended: null));
    }
  }

  Future<void> _start(String? noteId, Emitter<RecordingState> emit) async {
    if (_session != null || state.phase != RecordingPhase.idle) {
      emit(state.copyWith(problem: RecordingProblem.busy));
      return;
    }
    emit(
      state.copyWith(
        phase: RecordingPhase.starting,
        problem: null,
        saved: null,
        ended: null,
      ),
    );
    final started = await _repository.start(noteId: noteId);
    started.fold(
      (error) => emit(
        state.copyWith(
          phase: RecordingPhase.idle,
          problem: _problemOf(error),
        ),
      ),
      (session) {
        _session = session;
        _timer = Timer.periodic(_tick, (_) {
          if (!isClosed) {
            add(const RecordingEvent.ticked());
          }
        });
        emit(
          state.copyWith(
            phase: RecordingPhase.recording,
            seconds: 0,
            level: 0,
            paused: false,
            noteId: noteId,
          ),
        );
      },
    );
  }

  Future<void> _onTick(Emitter<RecordingState> emit) async {
    final session = _session;
    if (session == null || state.phase != RecordingPhase.recording) {
      return;
    }
    final status = session.status;
    if (status.ended != null) {
      // The cap, the microphone or the disk ended it: keep what it has.
      emit(state.copyWith(ended: status.ended));
      await _stop(emit);
      return;
    }
    emit(
      state.copyWith(
        seconds: status.seconds,
        level: status.level.clamp(0, 1),
        paused: status.paused,
        warned: status.warned,
      ),
    );
  }

  Future<void> _stop(Emitter<RecordingState> emit) async {
    final session = _session;
    if (session == null) {
      return;
    }
    _timer?.cancel();
    emit(state.copyWith(phase: RecordingPhase.saving));
    final result = await session.stop();
    _session = null;
    result.fold(
      (error) => emit(
        state.copyWith(
          phase: RecordingPhase.idle,
          problem: RecordingProblem.saveFailed,
        ),
      ),
      (saved) => emit(
        state.copyWith(
          phase: RecordingPhase.idle,
          seconds: 0,
          level: 0,
          paused: false,
          warned: false,
          saved: saved,
        ),
      ),
    );
    // A save that failed leaves the audio in its folder for Recover.
    if (result.isLeft()) {
      add(const RecordingEvent.foundChecked());
    }
  }

  Future<void> _discard(Emitter<RecordingState> emit) async {
    final session = _session;
    if (session == null) {
      return;
    }
    _timer?.cancel();
    emit(state.copyWith(phase: RecordingPhase.saving));
    await session.discard();
    _session = null;
    emit(
      state.copyWith(
        phase: RecordingPhase.idle,
        seconds: 0,
        level: 0,
        paused: false,
        warned: false,
      ),
    );
  }

  Future<void> _recover(String id, Emitter<RecordingState> emit) async {
    final result = await _repository.recover(id);
    result.fold(
      (error) => emit(state.copyWith(problem: RecordingProblem.saveFailed)),
      (saved) => emit(
        state.copyWith(
          saved: saved,
          found: <RecordingFound>[
            for (final f in state.found)
              if (f.id != id) f,
          ],
        ),
      ),
    );
  }

  static RecordingProblem _problemOf(ApplicationError error) => switch (error) {
    LockedError() => RecordingProblem.locked,
    NotFoundError() => RecordingProblem.noteGone,
    _ => RecordingProblem.cannotStart,
  };

  @override
  Future<void> close() {
    _timer?.cancel();
    return super.close();
  }
}
