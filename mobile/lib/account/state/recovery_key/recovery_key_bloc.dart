// Kanso — recovery_key_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/models/repos/abs/account_repository.dart';

part 'recovery_key_bloc.freezed.dart';
part 'recovery_key_event.dart';
part 'recovery_key_state.dart';

/// The recovery key of an account to write down, and the person saying they
/// have: the key is kept on this phone until they do.
@injectable
class RecoveryKeyBloc extends Bloc<RecoveryKeyEvent, RecoveryKeyState> {
  RecoveryKeyBloc(this._account) : super(const RecoveryKeyState()) {
    on<RecoveryKeyStartedEvent>(_onStarted, transformer: restartable());
    on<RecoveryKeyConfirmedChangedEvent>(_onConfirmedChanged);
    on<RecoveryKeyDoneEvent>(_onDone, transformer: droppable());
  }

  final AccountRepository _account;

  Future<void> _onStarted(
    RecoveryKeyStartedEvent event,
    Emitter<RecoveryKeyState> emit,
  ) async {
    final known = event.key;
    if (known != null) {
      emit(RecoveryKeyState(status: LoadStatus.ready, key: known));
      return;
    }
    emit(const RecoveryKeyState());
    final result = await _account.pendingRecoveryKey();
    emit(
      result.fold(
        (error) => RecoveryKeyState(status: LoadStatus.failed, error: error),
        (found) => RecoveryKeyState(status: LoadStatus.ready, key: found),
      ),
    );
  }

  void _onConfirmedChanged(
    RecoveryKeyConfirmedChangedEvent event,
    Emitter<RecoveryKeyState> emit,
  ) {
    emit(state.copyWith(confirmed: event.confirmed));
  }

  Future<void> _onDone(
    RecoveryKeyDoneEvent event,
    Emitter<RecoveryKeyState> emit,
  ) async {
    if (state.key != null && !state.confirmed) {
      return;
    }
    emit(state.copyWith(saving: true, error: null));
    final result = await _account.recoveryKeySaved();
    emit(
      result.fold(
        (error) => state.copyWith(saving: false, error: error),
        (_) => state.copyWith(saving: false, done: true),
      ),
    );
  }
}
