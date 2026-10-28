// Kanso — plugin_device_lock_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';
import 'package:kanso_lock/kanso_lock.dart';

import '../../../../core/models/data/application_error.dart';
import '../../../../core/utils/kanso_error_mapping.dart';
import '../abs/device_lock_repository.dart';

/// [DeviceLockRepository] over the `kanso_lock` plugin.
@LazySingleton(as: DeviceLockRepository)
class PluginDeviceLockRepository implements DeviceLockRepository {
  @override
  Future<Either<ApplicationError, BiometricKind>> availability() =>
      attempt(KansoLockPlugin.availability);

  @override
  Future<Either<ApplicationError, BiometricOutcome>> enroll(
    int ticket,
    String libraryId,
    String reason,
  ) => attempt(() => KansoLockPlugin.enroll(ticket, libraryId, reason));

  @override
  Future<Either<ApplicationError, BiometricOutcome>> unlock(
    int ticket,
    String libraryId,
    String reason,
  ) => attempt(() => KansoLockPlugin.unlock(ticket, libraryId, reason));

  @override
  Future<Either<ApplicationError, Unit>> forget(String libraryId) =>
      attempt(() async {
        await KansoLockPlugin.forget(libraryId);
        return unit;
      });

  @override
  Future<Either<ApplicationError, Unit>> setSecure({required bool on}) =>
      attempt(() async {
        await KansoLockPlugin.setSecure(on: on);
        return unit;
      });
}
