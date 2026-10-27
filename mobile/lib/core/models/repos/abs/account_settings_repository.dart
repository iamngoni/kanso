// Kanso — account_settings_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../bridge/api/types.dart';
import '../../data/application_error.dart';

/// The signed-in account's page: its devices and ways in, its password and
/// recovery key, signing out and deleting it. Each change that asks for a
/// proof takes the current password.
abstract interface class AccountSettingsRepository {
  Future<Either<ApplicationError, AccountDetails>> details();

  /// Signs another device out and gives the page again.
  Future<Either<ApplicationError, AccountDetails>> revokeDevice(
    String deviceId,
  );

  /// Signs this phone out; a locked sign-in ends too.
  Future<Either<ApplicationError, Unit>> signOut();

  /// Changes the password; the new recovery key is kept until it is saved.
  Future<Either<ApplicationError, RecoveryKeyText>> changePassword({
    required String current,
    required String newPassword,
  });

  Future<Either<ApplicationError, RecoveryKeyText>> newRecoveryKey(
    String currentPassword,
  );

  Future<Either<ApplicationError, Unit>> deleteAccount({
    required String password,
    required String typedUsername,
  });

  Future<Either<ApplicationError, AccountDetails>> addPasskey(
    String currentPassword,
  );

  Future<Either<ApplicationError, AccountDetails>> removePasskey({
    required String id,
    required String currentPassword,
  });

  /// The steps of linking [provider], until the flow ends.
  Stream<ProviderEvent> linkProvider({
    required ProviderKind provider,
    required String currentPassword,
  });

  Future<Either<ApplicationError, AccountDetails>> unlinkProvider({
    required ProviderKind provider,
    required String currentPassword,
  });

  /// The recovery key made here that has not been confirmed saved.
  Future<Either<ApplicationError, RecoveryKeyText?>> pendingRecoveryKey();
}
