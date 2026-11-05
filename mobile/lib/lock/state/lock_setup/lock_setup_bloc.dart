// Kanso — lock_setup_bloc.dart
// Created 2026-10-27.
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
import '../../models/data/lock_form_status.dart';
import '../../models/data/lock_mode.dart';
import '../../models/data/lock_target.dart';
import '../../models/data/password_check.dart';
import '../../utils/check_new_password.dart';
import '../../utils/lock_mode_for.dart';

part 'lock_setup_bloc.freezed.dart';
part 'lock_setup_event.dart';
part 'lock_setup_state.dart';

/// Locking a note or a notebook the desktop's three ways: the first lock
/// makes the lock password (typed twice), closed notes ask for it, open
/// notes only confirm.
@injectable
class LockSetupBloc extends Bloc<LockSetupEvent, LockSetupState> {
  LockSetupBloc(this._session) : super(const LockSetupState()) {
    on<LockSetupStartedEvent>(_onStarted, transformer: restartable());
    on<LockSetupSubmittedEvent>(_onSubmitted, transformer: droppable());
    on<LockSetupConfirmedEvent>(_onConfirmed, transformer: droppable());
    on<LockSetupProgressedEvent>(_onProgressed);
    on<LockSetupCancelledEvent>(_onCancelled, transformer: droppable());
  }

  final SessionRepository _session;

  Future<void> _onStarted(
    LockSetupStartedEvent event,
    Emitter<LockSetupState> emit,
  ) async {
    emit(LockSetupState(target: event.target));
    final status = await _session.lockStatus();
    status.fold(
      (error) => emit(
        state.copyWith(status: LockFormStatus.failed, error: error),
      ),
      (found) => emit(
        state.copyWith(
          mode: lockModeFor(found),
          status: LockFormStatus.ready,
        ),
      ),
    );
    // What it would seal, for the dialog's line about the attachments. A
    // failure to count leaves the line out.
    final target = event.target;
    final counted = switch (target) {
      NoteLockTarget(:final id) => await _session.noteLockSummary(id),
      NotebookLockTarget(:final id) => await _session.notebookLockSummary(id),
    };
    final summary = counted.toOption().toNullable();
    if (summary != null) {
      emit(state.copyWith(summary: summary));
    }
  }

  void _onProgressed(
    LockSetupProgressedEvent event,
    Emitter<LockSetupState> emit,
  ) => emit(state.copyWith(progress: event.progress));

  Future<void> _onCancelled(
    LockSetupCancelledEvent event,
    Emitter<LockSetupState> emit,
  ) async {
    if (state.status != LockFormStatus.working || state.cancelling) {
      return;
    }
    emit(state.copyWith(cancelling: true));
    await _session.cancelLock();
  }

  /// Locks the target while following how far sealing its files has come.
  Future<Either<ApplicationError, int>> _watched({String? password}) async {
    final listening = _session.lockProgress().listen((progress) {
      if (!isClosed) {
        add(LockSetupEvent.progressed(progress));
      }
    });
    try {
      return await _lock(password: password);
    } finally {
      unawaited(listening.cancel());
    }
  }

  Future<void> _onSubmitted(
    LockSetupSubmittedEvent event,
    Emitter<LockSetupState> emit,
  ) async {
    if (state.mode == LockMode.setPassword) {
      final check = checkNewPassword(event.password, event.again);
      if (check != PasswordCheck.ok) {
        emit(
          state.copyWith(
            status: check == PasswordCheck.mismatch
                ? LockFormStatus.mismatch
                : LockFormStatus.tooShort,
          ),
        );
        return;
      }
    }
    emit(
      state.copyWith(
        status: LockFormStatus.working,
        error: null,
        progress: null,
        cancelling: false,
      ),
    );
    final result = await _watched(password: event.password);
    final copied = await _copiesOf(result);
    result.fold(
      (error) => emit(
        // Cancel stopped it: the note waits to be finished or undone.
        state.cancelling
            ? state.copyWith(status: LockFormStatus.stopped, error: error)
            // A password that can't be the lock password is a wrong one.
            : error is AuthError || error is InvalidError
            ? state.copyWith(status: LockFormStatus.wrongPassword)
            : state.copyWith(status: LockFormStatus.failed, error: error),
      ),
      (count) => emit(
        state.copyWith(
          status: LockFormStatus.done,
          count: count,
          copyNotice: copied,
        ),
      ),
    );
  }

  Future<void> _onConfirmed(
    LockSetupConfirmedEvent event,
    Emitter<LockSetupState> emit,
  ) async {
    emit(
      state.copyWith(
        status: LockFormStatus.working,
        error: null,
        progress: null,
        cancelling: false,
      ),
    );
    final result = await _watched();
    final copied = await _copiesOf(result);
    result.fold(
      (error) => emit(
        // Cancel stopped it: the note waits to be finished or undone.
        state.cancelling
            ? state.copyWith(status: LockFormStatus.stopped, error: error)
            // The notes closed meanwhile: the password is needed after all.
            : error is LockedError
            ? state.copyWith(
                mode: LockMode.askPassword,
                status: LockFormStatus.ready,
              )
            : state.copyWith(status: LockFormStatus.failed, error: error),
      ),
      (count) => emit(
        state.copyWith(
          status: LockFormStatus.done,
          count: count,
          copyNotice: copied,
        ),
      ),
    );
  }

  /// The files a finished lock copied, to be said once; null when it
  /// failed or copied none.
  Future<LockCopyNotice?> _copiesOf(
    Either<ApplicationError, int> result,
  ) async => result.isRight()
      ? (await _session.takeCopyNotice()).toOption().toNullable()
      : null;

  /// Locks the target, with [password] or with the open session's key; the
  /// number of notes sealed (1 for a note).
  Future<Either<ApplicationError, int>> _lock({String? password}) async {
    final target = state.target;
    return switch (target) {
      null => const Left<ApplicationError, int>(
        ApplicationError.unexpected(message: 'Nothing to lock.'),
      ),
      NoteLockTarget(:final id) =>
        (password == null
                ? await _session.lockNoteWithSession(id)
                : await _session.lockNote(id, password))
            .map((Saved _) => 1),
      NotebookLockTarget(:final id) => await _session.lockNotebook(
        id,
        password: password,
      ),
    };
  }
}
