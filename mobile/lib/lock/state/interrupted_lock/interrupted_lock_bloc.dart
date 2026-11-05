// Kanso — interrupted_lock_bloc.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:dartz/dartz.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/repos/abs/session_repository.dart';

part 'interrupted_lock_bloc.freezed.dart';
part 'interrupted_lock_event.dart';
part 'interrupted_lock_state.dart';

/// The bar above a note whose lock stopped between its pictures and files
/// (Cancel, or the app quit): finish it, or undo it. When the session holds
/// no key for the lock the lock password is asked first. It also says, once,
/// that notes locked before this update seal their files when opened.
@injectable
class InterruptedLockBloc
    extends Bloc<InterruptedLockEvent, InterruptedLockState> {
  InterruptedLockBloc(this._session) : super(const InterruptedLockState()) {
    on<InterruptedLockCheckedEvent>(_onChecked, transformer: restartable());
    on<InterruptedLockFinishedEvent>(_onFinished, transformer: droppable());
    on<InterruptedLockUndoneEvent>(_onUndone, transformer: droppable());
    on<InterruptedLockPasswordDeclinedEvent>(
      (event, emit) => emit(
        state.copyWith(
          stage: InterruptedStage.stopped,
          action: null,
          error: null,
        ),
      ),
    );
    on<InterruptedLockProgressedEvent>(
      (event, emit) => emit(state.copyWith(progress: event.progress)),
    );
    on<InterruptedLockNoticeShownEvent>(
      (event, emit) => emit(state.copyWith(filesNotice: false)),
    );
  }

  final SessionRepository _session;

  Future<void> _onChecked(
    InterruptedLockCheckedEvent event,
    Emitter<InterruptedLockState> emit,
  ) async {
    // Finishing or undoing writes the note, which checks it again: that
    // check must not bring the buttons back while the work runs, or Undo
    // could start in the middle of Finish.
    if (state.stage == InterruptedStage.working &&
        state.noteId == event.noteId) {
      return;
    }
    // Opening a locked note may have sealed its files: ask after it opened.
    final notice = await _session.takeSealedFilesNotice();
    final say = notice.fold((_) => false, (said) => said);
    final found = await _session.interruptedLock(event.noteId);
    found.fold(
      (error) => emit(
        state.copyWith(
          noteId: event.noteId,
          stage: InterruptedStage.none,
          filesNotice: state.filesNotice || say,
        ),
      ),
      (stopped) => emit(
        state.copyWith(
          noteId: event.noteId,
          stage: stopped == null
              ? InterruptedStage.none
              : InterruptedStage.stopped,
          files: stopped?.files ?? 0,
          sealed: stopped?.sealed ?? 0,
          action: null,
          error: null,
          progress: null,
          filesNotice: state.filesNotice || say,
        ),
      ),
    );
  }

  Future<void> _onFinished(
    InterruptedLockFinishedEvent event,
    Emitter<InterruptedLockState> emit,
  ) => _run(
    InterruptedAction.finish,
    event.password,
    emit,
    (id, password) async =>
        (await _session.finishLocking(id, password: password)).map((_) => unit),
  );

  Future<void> _onUndone(
    InterruptedLockUndoneEvent event,
    Emitter<InterruptedLockState> emit,
  ) => _run(
    InterruptedAction.undo,
    event.password,
    emit,
    (id, password) => _session.undoLocking(id, password: password),
  );

  Future<void> _run(
    InterruptedAction action,
    String? password,
    Emitter<InterruptedLockState> emit,
    Future<Either<ApplicationError, Unit>> Function(String id, String? password)
    work,
  ) async {
    // Finish and Undo each have their own handler: one never starts while
    // the other works.
    if (state.stage == InterruptedStage.working) {
      return;
    }
    emit(
      state.copyWith(
        stage: InterruptedStage.working,
        action: action,
        error: null,
        progress: null,
      ),
    );
    final listening = _session.lockProgress().listen((progress) {
      if (!isClosed) {
        add(InterruptedLockEvent.progressed(progress));
      }
    });
    final Either<ApplicationError, Unit> result;
    try {
      result = await work(state.noteId, password);
    } finally {
      unawaited(listening.cancel());
    }
    emit(
      result.fold(
        (error) => switch (error) {
          LockedError() => state.copyWith(
            stage: InterruptedStage.needsPassword,
            action: action,
          ),
          AuthError() || InvalidError() when password != null => state.copyWith(
            stage: InterruptedStage.wrongPassword,
            action: action,
          ),
          _ => state.copyWith(
            stage: InterruptedStage.failed,
            action: action,
            error: error,
          ),
        },
        (_) => state.copyWith(
          stage: InterruptedStage.none,
          action: null,
          progress: null,
        ),
      ),
    );
  }
}
