// Kanso — change_password_bloc.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/models/data/application_error.dart';
import '../../../core/models/repos/abs/session_repository.dart';
import '../../models/data/lock_form_status.dart';
import '../../models/data/password_check.dart';
import '../../utils/check_new_password.dart';

part 'change_password_bloc.freezed.dart';
part 'change_password_event.dart';
part 'change_password_state.dart';

/// Changing the lock password: every locked note is sealed again with the
/// new one; those the current password doesn't open keep theirs and are
/// counted.
@injectable
class ChangePasswordBloc
    extends Bloc<ChangePasswordEvent, ChangePasswordState> {
  ChangePasswordBloc(this._session) : super(const ChangePasswordState()) {
    on<ChangePasswordSubmittedEvent>(_onSubmitted, transformer: droppable());
  }

  final SessionRepository _session;

  Future<void> _onSubmitted(
    ChangePasswordSubmittedEvent event,
    Emitter<ChangePasswordState> emit,
  ) async {
    final check = checkNewPassword(event.next, event.again);
    if (check != PasswordCheck.ok) {
      emit(
        ChangePasswordState(
          status: check == PasswordCheck.mismatch
              ? LockFormStatus.mismatch
              : LockFormStatus.tooShort,
        ),
      );
      return;
    }
    emit(const ChangePasswordState(status: LockFormStatus.working));
    final result = await _session.changePassword(event.current, event.next);
    emit(
      result.fold(
        (error) => error is AuthError || error is InvalidError
            ? const ChangePasswordState(status: LockFormStatus.wrongPassword)
            : ChangePasswordState(status: LockFormStatus.failed, error: error),
        (kept) => ChangePasswordState(status: LockFormStatus.done, kept: kept),
      ),
    );
  }
}
