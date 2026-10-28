// Kanso — unlock_bloc.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';
import 'package:kanso_lock/kanso_lock.dart';

import '../../../core/models/data/application_error.dart';
import '../../../core/models/repos/abs/preferences_repository.dart';
import '../../../core/models/repos/abs/session_repository.dart';
import '../../models/data/lock_target.dart';
import '../../models/data/unlock_status.dart';
import '../../models/repos/abs/device_lock_repository.dart';
import '../../utils/biometric_kinds.dart';

part 'unlock_bloc.freezed.dart';
part 'unlock_event.dart';
part 'unlock_state.dart';

/// Opening the locked notes for a note or a notebook: with a biometric, or
/// with the lock password typed.
///
/// The biometric secret goes from the phone's plugin to the Rust core; here
/// only the outcome is known. After the first password unlock that could
/// have offered a biometric, the state says so once.
@injectable
class UnlockBloc extends Bloc<UnlockEvent, UnlockState> {
  UnlockBloc(this._session, this._device, this._preferences)
    : super(const UnlockState()) {
    on<UnlockGateOpenedEvent>(_onGateOpened, transformer: droppable());
    on<UnlockBiometricRequestedEvent>(
      (event, emit) => _biometric(event.reason, emit),
      transformer: droppable(),
    );
    on<UnlockPasswordSubmittedEvent>(
      _onPasswordSubmitted,
      transformer: droppable(),
    );
  }

  final SessionRepository _session;
  final DeviceLockRepository _device;
  final PreferencesRepository _preferences;

  /// A notebook whose list is being shown: open already, or opened by a
  /// biometric when they are on; otherwise the state says the password is
  /// to be asked ([UnlockStatus.unavailable]).
  Future<void> _onGateOpened(
    UnlockGateOpenedEvent event,
    Emitter<UnlockState> emit,
  ) async {
    final target = event.target;
    if (target is NotebookLockTarget) {
      final open = await _session.notebookIsOpen(target.id);
      if (open.getOrElse(() => false)) {
        emit(const UnlockState(status: UnlockStatus.opened));
        return;
      }
    }
    final status = await _session.lockStatus();
    if (status.fold((_) => false, (found) => found.biometricsOn)) {
      await _biometric(event.reason, emit);
    } else {
      emit(const UnlockState(status: UnlockStatus.unavailable));
    }
  }

  Future<void> _biometric(String reason, Emitter<UnlockState> emit) async {
    emit(const UnlockState(status: UnlockStatus.working));
    final ticket = await _session.beginBiometricUnlock();
    final id = await _session.libraryId();
    final failure =
        ticket.fold<ApplicationError?>((e) => e, (_) => null) ??
        id.fold<ApplicationError?>((e) => e, (_) => null);
    if (failure != null) {
      emit(UnlockState(status: UnlockStatus.failed, error: failure));
      return;
    }
    final number = ticket.getOrElse(() => null);
    if (number == null) {
      emit(const UnlockState(status: UnlockStatus.unavailable));
      return;
    }
    final outcome = await _device.unlock(
      number,
      id.getOrElse(() => ''),
      reason,
    );
    emit(
      outcome.fold(
        (error) => UnlockState(status: UnlockStatus.failed, error: error),
        (done) => UnlockState(
          status: switch (done) {
            BiometricOutcome.done => UnlockStatus.opened,
            BiometricOutcome.cancelled => UnlockStatus.cancelled,
            BiometricOutcome.invalidated => UnlockStatus.invalidated,
            BiometricOutcome.unavailable => UnlockStatus.unavailable,
            BiometricOutcome.failed => UnlockStatus.failed,
          },
        ),
      ),
    );
  }

  Future<void> _onPasswordSubmitted(
    UnlockPasswordSubmittedEvent event,
    Emitter<UnlockState> emit,
  ) async {
    emit(const UnlockState(status: UnlockStatus.working));
    final target = event.target;
    final result = switch (target) {
      NoteLockTarget(:final id) => await _session.unlockNote(
        id,
        event.password,
      ),
      NotebookLockTarget(:final id) => await _session.unlockNotebook(
        id,
        event.password,
      ),
    };
    await result.fold(
      (error) async => emit(
        // A text that can't be a lock password is a wrong one.
        error is InvalidError
            ? const UnlockState(status: UnlockStatus.wrongPassword)
            : UnlockState(status: UnlockStatus.failed, error: error),
      ),
      (opened) async {
        if (!opened) {
          emit(const UnlockState(status: UnlockStatus.wrongPassword));
          return;
        }
        emit(
          UnlockState(
            status: UnlockStatus.opened,
            offerBiometrics: await _shouldOffer(),
          ),
        );
      },
    );
  }

  /// Whether a biometric is to be offered now, which is once: when it can
  /// be turned on and has not been offered. Asking marks it as offered.
  Future<bool> _shouldOffer() async {
    final status = await _session.lockStatus();
    final kind = await _device.availability();
    final saved = await _preferences.load();
    final offer =
        status.fold((_) => false, (found) => !found.biometricsOn) &&
        kind.fold((_) => false, biometricsAvailable) &&
        saved.fold((_) => true, (found) => !found.biometricOfferSeen);
    if (offer) {
      await _preferences.update(
        (current) => current.copyWith(biometricOfferSeen: true),
      );
    }
    return offer;
  }
}
