// Kanso — rust_account_settings_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../bridge/api/account_settings.dart';
import '../../../bridge/api/types.dart';
import '../../../utils/cancelable_stream.dart';
import '../../../utils/kanso_error_mapping.dart';
import '../../data/application_error.dart';
import '../abs/account_settings_repository.dart';

/// [AccountSettingsRepository] over the Rust core's account settings.
@LazySingleton(as: AccountSettingsRepository)
class RustAccountSettingsRepository implements AccountSettingsRepository {
  RustAccountSettingsRepository(this._settings);

  final KansoAccountSettings _settings;

  @override
  Future<Either<ApplicationError, AccountDetails>> details() =>
      attempt(_settings.details);

  @override
  Future<Either<ApplicationError, AccountDetails>> revokeDevice(
    String deviceId,
  ) => attempt(() => _settings.revokeDevice(deviceId: deviceId));

  @override
  Future<Either<ApplicationError, Unit>> signOut() => attempt(() async {
    await _settings.signOut();
    return unit;
  });

  @override
  Future<Either<ApplicationError, RecoveryKeyText>> changePassword({
    required String current,
    required String newPassword,
  }) => attempt(
    () => _settings.changePassword(current: current, newPassword: newPassword),
  );

  @override
  Future<Either<ApplicationError, RecoveryKeyText>> newRecoveryKey(
    String currentPassword,
  ) =>
      attempt(() => _settings.newRecoveryKey(currentPassword: currentPassword));

  @override
  Future<Either<ApplicationError, Unit>> deleteAccount({
    required String password,
    required String typedUsername,
  }) => attempt(() async {
    await _settings.deleteAccount(
      password: password,
      typedUsername: typedUsername,
    );
    return unit;
  });

  @override
  Future<Either<ApplicationError, AccountDetails>> addPasskey(
    String currentPassword,
  ) => attempt(() => _settings.addPasskey(currentPassword: currentPassword));

  @override
  Future<Either<ApplicationError, AccountDetails>> removePasskey({
    required String id,
    required String currentPassword,
  }) => attempt(
    () => _settings.removePasskey(id: id, currentPassword: currentPassword),
  );

  @override
  Stream<ProviderEvent> linkProvider({
    required ProviderKind provider,
    required String currentPassword,
  }) => cancelableStream(
    _settings.linkProvider(
      provider: provider,
      currentPassword: currentPassword,
    ),
  );

  @override
  Future<Either<ApplicationError, AccountDetails>> unlinkProvider({
    required ProviderKind provider,
    required String currentPassword,
  }) => attempt(
    () => _settings.unlinkProvider(
      provider: provider,
      currentPassword: currentPassword,
    ),
  );

  @override
  Future<Either<ApplicationError, RecoveryKeyText?>> pendingRecoveryKey() =>
      attempt(_settings.pendingRecoveryKey);
}
