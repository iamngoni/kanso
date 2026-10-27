// Kanso — lock_remove_bloc.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/models/data/application_error.dart';
import '../../../core/models/repos/abs/session_repository.dart';
import '../../models/data/lock_target.dart';
import '../../models/data/remove_status.dart';

part 'lock_remove_bloc.freezed.dart';
part 'lock_remove_event.dart';
part 'lock_remove_state.dart';

/// Taking the lock off an open note or notebook.
@injectable
class LockRemoveBloc extends Bloc<LockRemoveEvent, LockRemoveState> {
  LockRemoveBloc(this._session) : super(const LockRemoveState()) {
    on<LockRemoveConfirmedEvent>(_onConfirmed, transformer: droppable());
  }

  final SessionRepository _session;

  Future<void> _onConfirmed(
    LockRemoveConfirmedEvent event,
    Emitter<LockRemoveState> emit,
  ) async {
    emit(const LockRemoveState(status: RemoveStatus.working));
    final target = event.target;
    final result = switch (target) {
      NoteLockTarget(:final id) => (await _session.removeNoteLock(
        id,
      )).map((_) => null),
      NotebookLockTarget(:final id) => (await _session.removeNotebookLock(
        id,
      )).map((_) => null),
    };
    emit(
      result.fold(
        (ApplicationError error) =>
            LockRemoveState(status: RemoveStatus.failed, error: error),
        (_) => const LockRemoveState(status: RemoveStatus.done),
      ),
    );
  }
}
