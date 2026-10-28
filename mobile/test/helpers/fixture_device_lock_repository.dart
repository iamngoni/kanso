// Kanso — fixture_device_lock_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/lock/models/repos/abs/device_lock_repository.dart';
import 'package:kanso_lock/kanso_lock.dart';

/// The phone's side of locked notes for widget and golden tests: the kind of
/// biometric it has, and how a check ends.
class FixtureDeviceLockRepository implements DeviceLockRepository {
  FixtureDeviceLockRepository({this.kind = BiometricKind.faceId});

  BiometricKind kind;
  BiometricOutcome outcome = BiometricOutcome.done;

  /// What the Rust core does when an unlock ends well: a test opens the lock
  /// session and sends its event here, as the core does on the plugin's
  /// answer.
  void Function()? onUnlock;
  final List<String> calls = <String>[];
  final List<bool> secure = <bool>[];

  @override
  Future<Either<ApplicationError, BiometricKind>> availability() async =>
      Right<ApplicationError, BiometricKind>(kind);

  @override
  Future<Either<ApplicationError, BiometricOutcome>> enroll(
    int ticket,
    String libraryId,
    String reason,
  ) async {
    calls.add('enroll:$ticket:$libraryId');
    return Right<ApplicationError, BiometricOutcome>(outcome);
  }

  @override
  Future<Either<ApplicationError, BiometricOutcome>> unlock(
    int ticket,
    String libraryId,
    String reason,
  ) async {
    calls.add('unlock:$ticket:$libraryId');
    if (outcome == BiometricOutcome.done) {
      onUnlock?.call();
    }
    return Right<ApplicationError, BiometricOutcome>(outcome);
  }

  @override
  Future<Either<ApplicationError, Unit>> forget(String libraryId) async {
    calls.add('forget:$libraryId');
    return const Right<ApplicationError, Unit>(unit);
  }

  @override
  Future<Either<ApplicationError, Unit>> setSecure({required bool on}) async {
    secure.add(on);
    return const Right<ApplicationError, Unit>(unit);
  }
}
