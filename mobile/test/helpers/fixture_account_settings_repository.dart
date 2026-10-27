// Kanso — fixture_account_settings_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

// ignore_for_file: prefer_interpolation_to_compose_strings

import 'package:dartz/dartz.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/account_settings_repository.dart';

import 'fixture_account_repository.dart';

/// An account that is signed in on two devices, with a password, a passkey
/// and GitHub not linked.
const AccountDetails sampleDetails = AccountDetails(
  username: 'ana',
  server: 'https://sync.kansoapp.app',
  devices: <DeviceRow>[
    DeviceRow(
      id: 'd1',
      title: 'Ana’s iPhone',
      thisDevice: true,
      lastSeenMs: 1792000000000,
    ),
    DeviceRow(
      id: 'd2',
      title: 'Ana’s MacBook',
      thisDevice: false,
      lastSeenMs: 1791500000000,
    ),
  ],
  storageLine: '850 items · 3.1 MB of 100 MB · attachments 14 MB of 500 MB',
  hasPassword: true,
  providers: <LinkedProviderRow>[
    LinkedProviderRow(provider: ProviderKind.apple, linked: true),
    LinkedProviderRow(provider: ProviderKind.google, linked: false),
    LinkedProviderRow(provider: ProviderKind.github, linked: false),
  ],
  passkeys: <PasskeyRow>[
    PasskeyRow(
      id: 'p1',
      label: 'iCloud Keychain',
      addedMs: 1790000000000,
      lastUsedMs: 1791000000000,
      canUnlock: true,
    ),
  ],
  recoveryKeyUnsaved: false,
);

/// An account settings service for widget tests: the details it is given,
/// every change recorded by name. Each change gives the details again.
class FixtureAccountSettingsRepository implements AccountSettingsRepository {
  FixtureAccountSettingsRepository({this.current = sampleDetails});

  /// The details the account has.
  AccountDetails current;

  /// What every call fails with, if it does.
  ApplicationError? failure;

  /// The changes asked for, by name.
  final List<String> calls = <String>[];

  Either<ApplicationError, T> _answer<T>(T value) {
    final error = failure;
    return error == null
        ? Right<ApplicationError, T>(value)
        : Left<ApplicationError, T>(error);
  }

  @override
  Future<Either<ApplicationError, AccountDetails>> details() async =>
      _answer(current);

  @override
  Future<Either<ApplicationError, AccountDetails>> revokeDevice(
    String deviceId,
  ) async {
    calls.add('revokeDevice ' + deviceId);
    return _answer(current);
  }

  @override
  Future<Either<ApplicationError, Unit>> signOut() async {
    calls.add('signOut');
    return _answer(unit);
  }

  @override
  Future<Either<ApplicationError, RecoveryKeyText>> changePassword({
    required String current,
    required String newPassword,
  }) async {
    calls.add('changePassword');
    return _answer(sampleRecoveryKey);
  }

  @override
  Future<Either<ApplicationError, RecoveryKeyText>> newRecoveryKey(
    String currentPassword,
  ) async {
    calls.add('newRecoveryKey');
    return _answer(sampleRecoveryKey);
  }

  @override
  Future<Either<ApplicationError, Unit>> deleteAccount({
    required String password,
    required String typedUsername,
  }) async {
    calls.add('deleteAccount ' + typedUsername);
    return _answer(unit);
  }

  @override
  Future<Either<ApplicationError, AccountDetails>> addPasskey(
    String currentPassword,
  ) async {
    calls.add('addPasskey');
    return _answer(current);
  }

  @override
  Future<Either<ApplicationError, AccountDetails>> removePasskey({
    required String id,
    required String currentPassword,
  }) async {
    calls.add('removePasskey ' + id);
    return _answer(current);
  }

  @override
  Stream<ProviderEvent> linkProvider({
    required ProviderKind provider,
    required String currentPassword,
  }) {
    calls.add('linkProvider ' + provider.name);
    return const Stream<ProviderEvent>.empty();
  }

  @override
  Future<Either<ApplicationError, AccountDetails>> unlinkProvider({
    required ProviderKind provider,
    required String currentPassword,
  }) async {
    calls.add('unlinkProvider ' + provider.name);
    return _answer(current);
  }

  @override
  Future<Either<ApplicationError, RecoveryKeyText?>>
  pendingRecoveryKey() async => _answer<RecoveryKeyText?>(null);
}
