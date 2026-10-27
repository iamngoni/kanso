// Kanso — biometrics_bloc.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';
import 'package:kanso_lock/kanso_lock.dart';

import '../../../core/models/data/application_error.dart';
import '../../../core/models/repos/abs/session_repository.dart';
import '../../models/data/biometrics_status.dart';
import '../../models/repos/abs/device_lock_repository.dart';

part 'biometrics_bloc.freezed.dart';
part 'biometrics_event.dart';
part 'biometrics_state.dart';

/// Turning biometric unlock on or off. Turning it on needs the notes open:
/// the Rust core makes a ticket, the phone checks the biometric and gives
/// the secret to the core; turning it off forgets the keys kept for it.
@injectable
class BiometricsBloc extends Bloc<BiometricsEvent, BiometricsState> {
  BiometricsBloc(this._session, this._device) : super(const BiometricsState()) {
    on<BiometricsTurnedOnEvent>(_onTurnedOn, transformer: droppable());
    on<BiometricsTurnedOffEvent>(_onTurnedOff, transformer: droppable());
  }

  final SessionRepository _session;
  final DeviceLockRepository _device;

  Future<void> _onTurnedOn(
    BiometricsTurnedOnEvent event,
    Emitter<BiometricsState> emit,
  ) async {
    emit(const BiometricsState(status: BiometricsStatus.working));
    final status = await _session.lockStatus();
    if (status.fold((_) => false, (found) => !found.unlocked)) {
      emit(const BiometricsState(status: BiometricsStatus.notesClosed));
      return;
    }
    final ticket = await _session.beginBiometricEnrollment();
    final id = await _session.libraryId();
    final failure =
        ticket.fold<ApplicationError?>((e) => e, (_) => null) ??
        id.fold<ApplicationError?>((e) => e, (_) => null);
    if (failure != null) {
      emit(
        failure is LockedError
            ? const BiometricsState(status: BiometricsStatus.notesClosed)
            : BiometricsState(status: BiometricsStatus.failed, error: failure),
      );
      return;
    }
    final outcome = await _device.enroll(
      ticket.getOrElse(() => 0),
      id.getOrElse(() => ''),
      event.reason,
    );
    emit(
      outcome.fold(
        (error) =>
            BiometricsState(status: BiometricsStatus.failed, error: error),
        (done) => BiometricsState(
          status: switch (done) {
            BiometricOutcome.done => BiometricsStatus.enabled,
            BiometricOutcome.cancelled => BiometricsStatus.cancelled,
            BiometricOutcome.invalidated ||
            BiometricOutcome.unavailable ||
            BiometricOutcome.failed => BiometricsStatus.failed,
          },
        ),
      ),
    );
  }

  Future<void> _onTurnedOff(
    BiometricsTurnedOffEvent event,
    Emitter<BiometricsState> emit,
  ) async {
    emit(const BiometricsState(status: BiometricsStatus.working));
    final id = await _session.libraryId();
    final off = await _session.turnOffBiometrics();
    final failure =
        off.fold<ApplicationError?>((e) => e, (_) => null) ??
        id.fold<ApplicationError?>((e) => e, (_) => null);
    if (failure != null) {
      emit(BiometricsState(status: BiometricsStatus.failed, error: failure));
      return;
    }
    await _device.forget(id.getOrElse(() => ''));
    emit(const BiometricsState(status: BiometricsStatus.disabled));
  }
}
