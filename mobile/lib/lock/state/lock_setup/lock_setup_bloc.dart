// Kanso — lock_setup_bloc.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

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
    emit(state.copyWith(status: LockFormStatus.working, error: null));
    final result = await _lock(password: event.password);
    result.fold(
      (error) => emit(
        // A password that can't be the lock password is a wrong one.
        error is AuthError || error is InvalidError
            ? state.copyWith(status: LockFormStatus.wrongPassword)
            : state.copyWith(status: LockFormStatus.failed, error: error),
      ),
      (count) => emit(
        state.copyWith(status: LockFormStatus.done, count: count),
      ),
    );
  }

  Future<void> _onConfirmed(
    LockSetupConfirmedEvent event,
    Emitter<LockSetupState> emit,
  ) async {
    emit(state.copyWith(status: LockFormStatus.working, error: null));
    final result = await _lock();
    result.fold(
      (error) => emit(
        // The notes closed meanwhile: the password is needed after all.
        error is LockedError
            ? state.copyWith(
                mode: LockMode.askPassword,
                status: LockFormStatus.ready,
              )
            : state.copyWith(status: LockFormStatus.failed, error: error),
      ),
      (count) => emit(
        state.copyWith(status: LockFormStatus.done, count: count),
      ),
    );
  }

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
