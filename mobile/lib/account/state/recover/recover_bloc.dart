// Kanso — recover_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/repos/abs/account_repository.dart';
import '../../../core/models/repos/abs/sync_repository.dart';
import '../../models/data/account_outcome.dart';
import '../../models/data/form_problem.dart';
import '../../utils/account_forms.dart';

part 'recover_bloc.freezed.dart';
part 'recover_event.dart';
part 'recover_state.dart';

/// Getting into an account with its recovery key and choosing a new
/// password: from a locked phone, which recovers its own account, or from a
/// signed-out one, which names it.
///
/// A recovery makes a new recovery key; it is shown next, whether or not the
/// question about the notes on this phone is answered first.
@injectable
class RecoverBloc extends Bloc<RecoverEvent, RecoverState> {
  RecoverBloc(this._account, this._sync) : super(const RecoverState()) {
    on<RecoverStartedEvent>(_onStarted, transformer: restartable());
    on<RecoverSubmittedEvent>(_onSubmitted, transformer: droppable());
    on<RecoverBindingAnsweredEvent>(
      _onBindingAnswered,
      transformer: droppable(),
    );
  }

  final AccountRepository _account;
  final SyncRepository _sync;

  Future<void> _onStarted(
    RecoverStartedEvent event,
    Emitter<RecoverState> emit,
  ) async {
    final server = event.server;
    if (server != null) {
      emit(RecoverState(server: server));
      return;
    }
    final found = await _sync.defaultServer();
    emit(RecoverState(server: found.fold((_) => '', (address) => address)));
  }

  Future<void> _onSubmitted(
    RecoverSubmittedEvent event,
    Emitter<RecoverState> emit,
  ) async {
    final problem = AccountForms.recoveryProblem(
      username: event.username,
      recoveryKey: event.recoveryKey,
      password: event.password,
      repeat: event.repeat,
    );
    if (problem != null) {
      emit(state.copyWith(problem: problem, error: null));
      return;
    }
    emit(
      state.copyWith(
        busy: true,
        problem: null,
        error: null,
        question: null,
        recoveryKey: null,
        outcome: null,
      ),
    );
    final result = await _account.recover(
      server: state.server,
      username: AccountForms.normalizeUsername(event.username)!,
      recoveryKey: event.recoveryKey,
      newPassword: event.password,
      deviceName: event.deviceName,
    );
    emit(
      result.fold(
        (error) => state.copyWith(busy: false, error: error),
        (made) => _after(
          made.state,
          key: made.recoveryKey.key.isEmpty ? null : made.recoveryKey,
        ),
      ),
    );
  }

  Future<void> _onBindingAnswered(
    RecoverBindingAnsweredEvent event,
    Emitter<RecoverState> emit,
  ) async {
    emit(state.copyWith(question: null, busy: true));
    final result = await _account.answerBinding(event.answer);
    emit(
      result.fold(
        (error) => state.copyWith(busy: false, error: error),
        (account) => _after(account, key: state.recoveryKey),
      ),
    );
  }

  RecoverState _after(AccountState account, {RecoveryKeyText? key}) {
    return switch (account) {
      AccountState_SignedIn() => state.copyWith(
        busy: false,
        recoveryKey: key,
        outcome: AccountOutcome.signedIn,
      ),
      AccountState_Locked() => state.copyWith(
        busy: false,
        recoveryKey: key,
        outcome: AccountOutcome.locked,
      ),
      AccountState_Question(:final field0) => state.copyWith(
        busy: false,
        recoveryKey: key,
        question: field0,
      ),
      AccountState_SignedOut() => state.copyWith(
        busy: false,
        recoveryKey: key,
        outcome: key == null ? null : AccountOutcome.signedOut,
      ),
    };
  }
}
