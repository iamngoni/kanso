// Kanso — device_lock_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:kanso_lock/kanso_lock.dart';

import '../../../../core/models/data/application_error.dart';

/// What the phone itself does for locked notes: the biometric check and the
/// cover over the app. The secrets go from the plugin to the Rust core; this
/// only learns how a check ended.
abstract interface class DeviceLockRepository {
  Future<Either<ApplicationError, BiometricKind>> availability();

  Future<Either<ApplicationError, BiometricOutcome>> enroll(
    int ticket,
    String libraryId,
    String reason,
  );

  Future<Either<ApplicationError, BiometricOutcome>> unlock(
    int ticket,
    String libraryId,
    String reason,
  );

  Future<Either<ApplicationError, Unit>> forget(String libraryId);

  /// Keeps the app out of screenshots and the app switcher while [on].
  Future<Either<ApplicationError, Unit>> setSecure({required bool on});
}
