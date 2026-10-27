// Kanso — lock_status_bloc.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:flutter/widgets.dart' show AppLifecycleState;
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';
import 'package:kanso_lock/kanso_lock.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/repos/abs/session_repository.dart';
import '../../models/repos/abs/device_lock_repository.dart';
import '../../utils/biometric_kinds.dart';

part 'lock_status_bloc.freezed.dart';
part 'lock_status_event.dart';
part 'lock_status_state.dart';

/// Whether the locked notes are open, whether a lock password exists and
/// whether biometrics can open them, kept up to date for every screen that
/// shows a lock, and the one that closes the notes when the app goes out of
/// sight.
@injectable
class LockStatusBloc extends Bloc<LockStatusEvent, LockStatusState> {
  LockStatusBloc(this._session, this._device) : super(const LockStatusState()) {
    on<LockStatusStartedEvent>(_onStarted, transformer: sequential());
    on<LockStatusRefreshedEvent>(
      (event, emit) => _read(emit),
      transformer: sequential(),
    );
    on<LockStatusLockedNowEvent>(_onLockedNow, transformer: sequential());
    on<LockStatusLifecycleChangedEvent>(
      _onLifecycleChanged,
      transformer: sequential(),
    );
  }

  final SessionRepository _session;
  final DeviceLockRepository _device;
  StreamSubscription<LockEvent>? _events;

  Future<void> _onStarted(
    LockStatusStartedEvent event,
    Emitter<LockStatusState> emit,
  ) async {
    await _events?.cancel();
    _events = _session.lockEvents().listen(
      (_) => add(const LockStatusEvent.refreshed()),
      onError: (Object _) {},
    );
    await _read(emit);
  }

  Future<void> _onLockedNow(
    LockStatusLockedNowEvent event,
    Emitter<LockStatusState> emit,
  ) async {
    await _session.lock();
    await _read(emit);
  }

  Future<void> _onLifecycleChanged(
    LockStatusLifecycleChangedEvent event,
    Emitter<LockStatusState> emit,
  ) async {
    // Leaving the app closes the notes; a system sheet over it (the
    // notification shade) only makes it inactive and does not.
    if (event.lifecycle == AppLifecycleState.hidden && state.unlocked) {
      await _session.lockForBackground();
      await _read(emit);
    }
  }

  Future<void> _read(Emitter<LockStatusState> emit) async {
    final (status, kind) = await (
      _session.lockStatus(),
      _device.availability(),
    ).wait;
    if (emit.isDone) {
      return;
    }
    final found = status.toOption().toNullable();
    emit(
      state.copyWith(
        loaded: true,
        hasLockPassword: found?.hasLockPassword ?? state.hasLockPassword,
        unlocked: found?.unlocked ?? state.unlocked,
        biometricsOn: found?.biometricsOn ?? state.biometricsOn,
        kind: kind.getOrElse(() => state.kind),
      ),
    );
  }

  @override
  Future<void> close() async {
    await _events?.cancel();
    return super.close();
  }
}
